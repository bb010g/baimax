use nom;
use nom::{ErrorKind, IResult};
use ast;

fn u8_char(input: &[u8], ch: u8) -> IResult<&[u8], u8> {
    if let Some(first) = input.first() {
        if *first == ch {
            IResult::Done(&input[1..], ch)
        } else {
            IResult::Error(error_position!(ErrorKind::Custom(100), input))
        }
    } else {
        IResult::Incomplete(nom::Needed::Size(1))
    }
}
macro_rules! specialize_u8_char {
    ($name:ident, $ch:expr) => { fn $name(input: &[u8]) -> IResult<&[u8], u8> {
        u8_char(input, $ch)
    } }
}
specialize_u8_char!(space_char, b' ');

named!(end_of_line, alt!(call!(nom::eol) | eof!()));
const FIELD_SEP_CHAR: u8 = b',';
specialize_u8_char!(field_sep_char, FIELD_SEP_CHAR);
const RECORD_SEP_CHAR: u8 = b'/';
specialize_u8_char!(record_sep_char, RECORD_SEP_CHAR);

named!(continuation, tag!(b"88,"));
enum FieldSep {
    Normal,
    Continuation,
}
named!(
    field_sep<FieldSep>,
    alt!(
        value!(FieldSep::Normal, field_sep_char) |
        value!(FieldSep::Continuation,
               tuple!(record_sep_char, call!(nom::eol), continuation))
    )
);
named!(record_sep<u8>, terminated!(record_sep_char, many0!(space_char)));
fn is_field_char(c: u8) -> bool {
    c != FIELD_SEP_CHAR && c != RECORD_SEP_CHAR
}
named!(field_inner, take_while1!(is_field_char));

pub fn text_start_char(input: &[u8]) -> IResult<&[u8], u8> {
    if input.is_empty() {
        IResult::Incomplete(nom::Needed::Size(1))
    } else {
        IResult::Done(&input[1..], input[0])
    }
}
pub fn is_text_char(c: u8) -> bool {
    c != b'\n' && c != b'\r'
}
named!(
    text_line<(&[u8], bool)>,
    tuple!(
        terminated!(take_while!(is_text_char), many0!(space_char)),
        map!(opt!(preceded!(call!(nom::eol), continuation)), |o: Option<_>| o.is_none())
    )
);
fn text_inner(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    use nom::InputLength;

    let ret;
    let mut res = Vec::new();
    let mut input = input;

    loop {
        if input.input_len() == 0 {
            ret = IResult::Done(input, res);
            break;
        }

        match text_line(input) {
            nom::IResult::Error(_) => {
                ret = IResult::Done(input, res);
                break;
            }
            nom::IResult::Incomplete(nom::Needed::Unknown) => {
                ret = IResult::Incomplete(nom::Needed::Unknown);
                break;
            }
            nom::IResult::Incomplete(nom::Needed::Size(i)) => {
                let size = i + input.input_len() - input.input_len();
                ret = IResult::Incomplete(nom::Needed::Size(size));
                break;
            }
            nom::IResult::Done(i, (o, stop)) => {
                // loop trip must always consume (otherwise infinite loops)
                if i == input {
                    ret = IResult::Error(error_position!(nom::ErrorKind::Many0, input));
                    break;
                }

                res.push(o);
                input = i;
                if stop {
                    ret = IResult::Done(input, res);
                    break;
                }
            }
        }
    }

    ret
}
//named!(
//    text_inner<Vec<&[u8]>>,
//    map!(
//        tuple!(
//            many0!(do_parse!(
//                chars: take_while!(is_text_char) >> many0!(space_char) >>
//                call!(nom::eol) >> continuation >>
//                (chars)
//            )),
//            take_while!(is_text_char)
//        ),
//        |(mut vec, lst): (Vec<&'a [u8]>, &'a [u8])| { if !lst.is_empty() { vec.push(last) } vec }
//    )
//);
named!(text<(u8, Vec<&[u8]>)>, tuple!(text_start_char, text_inner));

named!(
    distributed_avail_distribution_inner<ast::RawDistributedAvailDistribution>,
    do_parse!(
        days: field_inner >> field_sep >>
        amount: field_inner >>
        (ast::RawDistributedAvailDistribution {
            days,
            amount,
        })
    )
);

named!(
    funds_type_inner<ast::RawFundsType>,
    alt!(
        value!(ast::RawFundsType::Unknown, call!(u8_char, b'Z')) |
        value!(ast::RawFundsType::ImmediateAvail, call!(u8_char, b'0')) |
        value!(ast::RawFundsType::OneDayAvail, call!(u8_char, b'1')) |
        value!(ast::RawFundsType::TwoOrMoreDaysAvail, call!(u8_char, b'2')) |
        preceded!(call!(u8_char, b'S'), return_error!(ErrorKind::Custom(105), do_parse!(
            field_sep >>
            immediate: opt!(field_inner) >> field_sep >>
            one_day: opt!(field_inner) >> field_sep >>
            more_than_one_day: opt!(field_inner) >>
            (ast::RawFundsType::DistributedAvailS {
                immediate,
                one_day,
                more_than_one_day,
            })
        ))) |
        preceded!(call!(u8_char, b'V'), return_error!(ErrorKind::Custom(106), do_parse!(
            field_sep >>
            date: field_inner >> field_sep >>
            time: opt!(field_inner) >>
            (ast::RawFundsType::ValueDated {
                date,
                time,
            })
        ))) |
        preceded!(call!(u8_char, b'D'), return_error!(ErrorKind::Custom(107), do_parse!(
            field_sep >>
            num: field_inner >> field_sep >>
            dists: separated_nonempty_list!(field_sep, distributed_avail_distribution_inner) >>
            (ast::RawFundsType::DistributedAvailD {
                num,
                dists,
            })
        )))
    )
);

named!(
    account_info_inner<ast::RawAccountInfo>,
    do_parse!(
        type_code: opt!(field_inner) >> field_sep >>
        amount: opt!(field_inner) >> field_sep >>
        item_count: opt!(field_inner) >> field_sep >>
        funds_type: opt!(funds_type_inner) >>
        (ast::RawAccountInfo {
            type_code,
            amount,
            item_count,
            funds_type,
        })
    )
);

named!(
    pub record<ast::RawRecord>,
    alt!(
        preceded!(tag!(b"01"), return_error!(ErrorKind::Custom(1), do_parse!(
            field_sep >>
            sender_ident: field_inner >> field_sep >>
            receiver_ident: field_inner >> field_sep >>
            creation_date: field_inner >> field_sep >>
            creation_time: field_inner >> field_sep >>
            ident_num: field_inner >> field_sep >>
            physical_record_len: opt!(field_inner) >> field_sep >>
            block_size: opt!(field_inner) >> field_sep >>
            version_number: field_inner >>
            record_sep >>
            (ast::RawRecord::FileHeader(ast::RawFileHeader {
                sender_ident,
                receiver_ident,
                creation_date,
                creation_time,
                ident_num,
                physical_record_len,
                block_size,
                version_number,
            }))
        ))) |
        preceded!(tag!(b"02"), return_error!(ErrorKind::Custom(2), do_parse!(
            field_sep >>
            ultimate_receiver_ident: opt!(field_inner) >> field_sep >>
            originator_ident: opt!(field_inner) >> field_sep >>
            status: field_inner >> field_sep >>
            as_of_date: field_inner >> field_sep >>
            as_of_time: opt!(field_inner) >> field_sep >>
            currency: opt!(field_inner) >> field_sep >>
            as_of_date_mod: opt!(field_inner) >>
            record_sep >>
            (ast::RawRecord::GroupHeader(ast::RawGroupHeader {
                ultimate_receiver_ident,
                originator_ident,
                status,
                as_of_date,
                as_of_time,
                currency,
                as_of_date_mod,
            }))
        ))) |
        preceded!(tag!(b"03"), return_error!(ErrorKind::Custom(3), do_parse!(
            field_sep >>
            customer_account_num: field_inner >> field_sep >>
            currency: opt!(field_inner) >> field_sep >>
            infos: separated_nonempty_list!(field_sep, account_info_inner) >>
            record_sep >>
            (ast::RawRecord::AccountIdent(ast::RawAccountIdent {
                customer_account_num,
                currency,
                infos,
            }))
        ))) |
        preceded!(tag!(b"16"), return_error!(ErrorKind::Custom(16), do_parse!(
            field_sep >>
            type_code: field_inner >> field_sep >>
            amount: opt!(field_inner) >> field_sep >>
            funds_type: opt!(funds_type_inner) >> field_sep >>
            bank_ref_num: opt!(field_inner) >> field_sep >>
            customer_ref_num: opt!(field_inner) >> field_sep >>
            txt: alt!(value!(None, record_sep) | map!(text, Some)) >>
            (ast::RawRecord::TransactionDetail(ast::RawTransactionDetail {
                type_code,
                amount,
                funds_type,
                bank_ref_num,
                customer_ref_num,
                text: txt,
            }))
        ))) |
        preceded!(tag!(b"49"), return_error!(ErrorKind::Custom(49), do_parse!(
            field_sep >>
            control_total: field_inner >> field_sep >>
            records_num: field_inner >>
            record_sep >>
            (ast::RawRecord::AccountTrailer(ast::RawAccountTrailer {
                control_total,
                records_num,
            }))
        ))) |
        preceded!(tag!(b"98"), return_error!(ErrorKind::Custom(98), do_parse!(
            field_sep >>
            control_total: field_inner >> field_sep >>
            accounts_num: field_inner >> field_sep >>
            records_num: field_inner >>
            record_sep >>
            (ast::RawRecord::GroupTrailer(ast::RawGroupTrailer {
                control_total,
                accounts_num,
                records_num,
            }))
        ))) |
        preceded!(tag!(b"99"), return_error!(ErrorKind::Custom(99), do_parse!(
            field_sep >>
            control_total: field_inner >> field_sep >>
            groups_num: field_inner >> field_sep >>
            records_num: field_inner >>
            record_sep >>
            (ast::RawRecord::FileTrailer(ast::RawFileTrailer {
                control_total,
                groups_num,
                records_num,
            }))
        )))
    )
);

named!(
    pub file<Vec<ast::RawRecord>>,
    many0!(terminated!(record, end_of_line))
    //tap!(res: many0!(terminated!(record, end_of_line)) => {println!("{:?}", res)})
);
