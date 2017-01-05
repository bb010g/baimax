use nom;
use nom::{ErrorKind, IResult};
use ::ast;

named!(end_of_line, alt!(eof!() | call!(nom::eol)));
const FIELD_SEP_CHAR: u8 = b',';
named!(
    field_sep<u32>,
    alt_complete!(
        value!(0, tag!(",")) |
        value!(1, delimited!(tag!("/"), call!(nom::eol), tag!("88,")))
    )
);
const RECORD_SEP_CHAR: u8 = b'/';
named!(record_sep, tag!("/"));
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
    text_inner<Vec<&[u8]>>,
    map!(
        pair!(
            many0!(terminated!(take_while!(is_text_char), preceded!(nom::eol, tag!("88,")))),
            take_while!(is_text_char)
        ),
        |(mut vec, last): (Vec<&'a [u8]>, &'a [u8])| { if last.len() > 0 { vec.push(last) } vec }
    )
);
named!(
    text<(u8, Vec<&[u8]>)>,
    do_parse!(
        start: text_start_char >>
        rest: text_inner >>
        (start, rest)
    )
);

named!(
    distributed_avail_distribution_inner<ast::RawDistributedAvailDistribution>,
    do_parse!(
        days: field_inner >> field_sep >>
        amount: field_inner >>
        (ast::RawDistributedAvailDistribution {
            days: days,
            amount: amount,
        })
    )
);

named!(
    funds_type_inner<ast::RawFundsType>,
    alt_complete!(
        value!(ast::RawFundsType::Unknown, tag!("Z")) |
        value!(ast::RawFundsType::ImmediateAvail, tag!("0")) |
        value!(ast::RawFundsType::OneDayAvail, tag!("1")) |
        value!(ast::RawFundsType::TwoOrMoreDaysAvail, tag!("2")) |
        do_parse!(
            tag!("S") >> field_sep >>
            immediate: opt!(field_inner) >> field_sep >>
            one_day: opt!(field_inner) >> field_sep >>
            more_than_one_day: opt!(field_inner) >>
            (ast::RawFundsType::DistributedAvailS {
                immediate: immediate,
                one_day: one_day,
                more_than_one_day: more_than_one_day,
            })
        ) |
        do_parse!(
            tag!("V") >> field_sep >>
            date: field_inner >> field_sep >>
            time: opt!(field_inner) >>
            (ast::RawFundsType::ValueDated {
                date: date,
                time: time,
            })
        ) |
        do_parse!(
            tag!("D") >> field_sep >>
            num: field_inner >> field_sep >>
            dists: separated_nonempty_list!(field_sep, distributed_avail_distribution_inner) >>
            (ast::RawFundsType::DistributedAvailD {
                num: num,
                dists: dists,
            })
        )
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
            type_code: type_code,
            amount: amount,
            item_count: item_count,
            funds_type: funds_type,
        })
    )
);

named!(
    pub record<ast::RawRecord>,
    alt_complete!(
        preceded!(tag!("01"), return_error!(ErrorKind::Custom(1), do_parse!(
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
                sender_ident: sender_ident,
                receiver_ident: receiver_ident,
                creation_date: creation_date,
                creation_time: creation_time,
                ident_num: ident_num,
                physical_record_len: physical_record_len,
                block_size: block_size,
                version_number: version_number,
            }))
        ))) |
        preceded!(tag!("02"), return_error!(ErrorKind::Custom(2), do_parse!(
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
                ultimate_receiver_ident: ultimate_receiver_ident,
                originator_ident: originator_ident,
                status: status,
                as_of_date: as_of_date,
                as_of_time: as_of_time,
                currency: currency,
                as_of_date_mod: as_of_date_mod,
            }))
        ))) |
        preceded!(tag!("03"), return_error!(ErrorKind::Custom(3), do_parse!(
            field_sep >>
            customer_account_num: field_inner >> field_sep >>
            currency: opt!(field_inner) >> field_sep >>
            infos: separated_nonempty_list!(field_sep, account_info_inner) >>
            record_sep >>
            (ast::RawRecord::AccountIdent(ast::RawAccountIdent {
                customer_account_num: customer_account_num,
                currency: currency,
                infos: infos,
            }))
        ))) |
        preceded!(tag!("16"), return_error!(ErrorKind::Custom(16), do_parse!(
            field_sep >>
            type_code: field_inner >> field_sep >>
            amount: opt!(field_inner) >> field_sep >>
            funds_type: opt!(funds_type_inner) >> field_sep >>
            bank_ref_num: opt!(field_inner) >> field_sep >>
            customer_ref_num: opt!(field_inner) >> field_sep >>
            txt: alt_complete!(value!(None, record_sep) | map!(text, Some)) >>
            (ast::RawRecord::TransactionDetail(ast::RawTransactionDetail {
                type_code: type_code,
                amount: amount,
                funds_type: funds_type,
                bank_ref_num: bank_ref_num,
                customer_ref_num: customer_ref_num,
                text: txt,
            }))
        ))) |
        preceded!(tag!("49"), return_error!(ErrorKind::Custom(49), do_parse!(
            field_sep >>
            control_total: field_inner >> field_sep >>
            records_num: field_inner >>
            record_sep >>
            (ast::RawRecord::AccountTrailer(ast::RawAccountTrailer {
                control_total: control_total,
                records_num: records_num,
            }))
        ))) |
        preceded!(tag!("98"), return_error!(ErrorKind::Custom(98), do_parse!(
            field_sep >>
            control_total: field_inner >> field_sep >>
            accounts_num: field_inner >> field_sep >>
            records_num: field_inner >>
            record_sep >>
            (ast::RawRecord::GroupTrailer(ast::RawGroupTrailer {
                control_total: control_total,
                accounts_num: accounts_num,
                records_num: records_num,
            }))
        ))) |
        preceded!(tag!("99"), return_error!(ErrorKind::Custom(99), do_parse!(
            field_sep >>
            control_total: field_inner >> field_sep >>
            groups_num: field_inner >> field_sep >>
            records_num: field_inner >>
            tag!("/") >>
            (ast::RawRecord::FileTrailer(ast::RawFileTrailer {
                control_total: control_total,
                groups_num: groups_num,
                records_num: records_num,
            }))
        )))
    )
);

named!(
    pub file<Vec<ast::RawRecord>>,
    many0!(terminated!(record, end_of_line))
);
