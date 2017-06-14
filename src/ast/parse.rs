use super::*;

pub trait Parsed {
    type Raw;
    type Parsed;
    type Field;
    type Err;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>>;
}

#[derive(Debug)]
pub enum ParseError<T: Parsed + ?Sized> {
    Utf8(T::Field, str::Utf8Error),
    Int(T::Field, num::ParseIntError),
    Format(T::Field),
    Error(T::Field, T::Err),
}
fn parse_str<T: Parsed>(i: &[u8], f: T::Field) -> Result<&str, ParseError<T>> {
    str::from_utf8(i).map_err(|e| ParseError::Utf8(f, e))
}
fn parse_from<T, P, F>(i: &str, f: T::Field, fs: F) -> Result<P, ParseError<T>>
where
    T: Parsed,
    T::Field: Copy,
    P: str::FromStr,
    F: FnOnce(P::Err) -> T::Err,
{
    i.parse::<P>().map_err(|e| ParseError::Error(f, fs(e)))
}
fn parse_int<T, P>(i: &str, f: T::Field) -> Result<P, ParseError<T>>
where
    T: Parsed,
    T::Field: Copy,
    P: str::FromStr<Err = num::ParseIntError>,
{
    i.parse::<P>().map_err(|e| ParseError::Int(f, e))
}
fn parse_strfrom<T, P, F>(i: &[u8], f: T::Field, fs: F) -> Result<P, ParseError<T>>
where
    T: Parsed,
    T::Field: Copy,
    P: str::FromStr,
    F: FnOnce(P::Err) -> T::Err,
{
    parse_str(i, f).and_then(|s| parse_from(s, f, fs))
}
fn parse_strint<T, P>(i: &[u8], f: T::Field) -> Result<P, ParseError<T>>
where
    T: Parsed,
    T::Field: Copy,
    P: str::FromStr<Err = num::ParseIntError>,
{
    parse_str(i, f).and_then(|s| parse_int(s, f))
}
fn parse_optstr<T: Parsed>(i: Option<&[u8]>, f: T::Field) -> Result<Option<&str>, ParseError<T>> {
    i.map_or(Ok(None), |s| parse_str(s, f).map(Some))
}
fn parse_optstrfrom<T, P, F>(
    i: Option<&[u8]>,
    f: T::Field,
    fs: F,
) -> Result<Option<P>, ParseError<T>>
where
    T: Parsed,
    T::Field: Copy,
    P: str::FromStr,
    F: FnOnce(P::Err) -> T::Err,
{
    i.map_or(Ok(None), |s| parse_strfrom(s, f, fs).map(Some))
}
fn parse_optstrint<T, P>(i: Option<&[u8]>, f: T::Field) -> Result<Option<P>, ParseError<T>>
where
    T: Parsed,
    T::Field: Copy,
    P: str::FromStr<Err = num::ParseIntError>,
{
    i.map_or(Ok(None), |s| parse_strint(s, f).map(Some))
}

#[derive(Debug)]
pub enum RecordError<'a> {
    FileHeader(ParseError<FileHeader<'a>>),
    GroupHeader(ParseError<GroupHeader<'a>>),
    AccountIdent(ParseError<AccountIdent<'a>>),
    TransactionDetail(ParseError<TransactionDetail<'a>>),
    AccountTrailer(ParseError<AccountTrailer<'a>>),
    GroupTrailer(ParseError<GroupTrailer<'a>>),
    FileTrailer(ParseError<FileTrailer<'a>>),
}
impl<'a> Parsed for Record<'a> {
    type Raw = RawRecord<'a>;
    type Parsed = ParsedRecord<'a>;
    type Field = RecordField;
    type Err = RecordError<'a>;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::ParseError as PE;
        use self::RecordField as F;
        use self::RecordError as E;

        use self::RawRecord as R;
        use self::ParsedRecord as P;
        Ok(match *raw {
            R::FileHeader(ref fh) => {
                P::FileHeader(FileHeader::parse(fh).map_err(|e| {
                    PE::Error(F::FileHeader, E::FileHeader(e))
                })?)
            }
            R::GroupHeader(ref gh) => {
                P::GroupHeader(GroupHeader::parse(gh).map_err(|e| {
                    PE::Error(F::GroupHeader, E::GroupHeader(e))
                })?)
            }
            R::AccountIdent(ref ai) => {
                P::AccountIdent(AccountIdent::parse(ai).map_err(|e| {
                    PE::Error(F::AccountIdent, E::AccountIdent(e))
                })?)
            }
            R::TransactionDetail(ref td) => {
                P::TransactionDetail(TransactionDetail::parse(td).map_err(|e| {
                    PE::Error(F::TransactionDetail, E::TransactionDetail(e))
                })?)
            }
            R::AccountTrailer(ref at) => {
                P::AccountTrailer(AccountTrailer::parse(at).map_err(|e| {
                    PE::Error(F::AccountTrailer, E::AccountTrailer(e))
                })?)
            }
            R::GroupTrailer(ref gt) => {
                P::GroupTrailer(GroupTrailer::parse(gt).map_err(|e| {
                    PE::Error(F::GroupTrailer, E::GroupTrailer(e))
                })?)
            }
            R::FileTrailer(ref ft) => {
                P::FileTrailer(FileTrailer::parse(ft).map_err(|e| {
                    PE::Error(F::FileTrailer, E::FileTrailer(e))
                })?)
            }
        })
    }
}

#[derive(Debug)]
pub enum FileHeaderError {
    Date(DateError),
    Time(TimeError),
}
impl<'a> Parsed for FileHeader<'a> {
    type Raw = RawFileHeader<'a>;
    type Parsed = ParsedFileHeader<'a>;
    type Field = FileHeaderField;
    type Err = FileHeaderError;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::ParseError as PE;
        use self::FileHeaderField as F;
        use self::FileHeaderError as E;
        Ok(ParsedFileHeader {
            sender_ident: parse_str(raw.sender_ident, F::SenderIdent)?,
            receiver_ident: parse_str(raw.receiver_ident, F::ReceiverIdent)?,
            creation_date: parse_strfrom(raw.creation_date, F::CreationDate, E::Date)?,
            creation_time: parse_strfrom(raw.creation_time, F::CreationTime, E::Time)?,
            ident_num: parse_strint(raw.ident_num, F::IdentNum)?,
            physical_record_len: parse_optstrint(raw.physical_record_len, F::PhysicalRecordLen)?,
            block_size: parse_optstrint(raw.block_size, F::BlockSize)?,
            version_number: if parse_str(raw.version_number, F::VersionNumber)? == "2" {
                ()
            } else {
                return Err(PE::Format(F::VersionNumber));
            },
        })
    }
}

#[derive(Debug)]
pub enum GroupHeaderError {
    Date(DateError),
    Time(TimeError),
}
impl<'a> Parsed for GroupHeader<'a> {
    type Raw = RawGroupHeader<'a>;
    type Parsed = ParsedGroupHeader<'a>;
    type Field = GroupHeaderField;
    type Err = GroupHeaderError;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::GroupHeaderField as F;
        use self::GroupHeaderError as E;
        Ok(ParsedGroupHeader {
            ultimate_receiver_ident: parse_optstr(
                raw.ultimate_receiver_ident,
                F::UltimateReceiverIdent,
            )?,
            originator_ident: parse_optstr(raw.originator_ident, F::OriginatorIdent)?,
            status: parse_strint(raw.status, F::Status)?,
            as_of_date: parse_strfrom(raw.as_of_date, F::AsOfDate, E::Date)?,
            as_of_time: parse_optstrfrom(raw.as_of_time, F::AsOfTime, E::Time)?,
            currency: parse_optstr(raw.currency, F::AsOfTime)?,
            as_of_date_mod: parse_optstrint(raw.as_of_date_mod, F::AsOfDateMod)?,
        })
    }
}

#[derive(Debug)]
pub enum AccountIdentError<'a> {
    Info(usize, ParseError<AccountInfo<'a>>),
}
impl<'a> Parsed for AccountIdent<'a> {
    type Raw = RawAccountIdent<'a>;
    type Parsed = ParsedAccountIdent<'a>;
    type Field = AccountIdentField;
    type Err = AccountIdentError<'a>;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::ParseError as PE;
        use self::AccountIdentField as F;
        use self::AccountIdentError as E;
        Ok(ParsedAccountIdent {
            customer_account_num: parse_str(raw.customer_account_num, F::CustomerAccountNum)?,
            currency: parse_optstr(raw.currency, F::Currency)?,
            infos: {
                let mut p = Vec::with_capacity(raw.infos.len());
                for (i, info) in raw.infos.iter().enumerate() {
                    p.push(Parsed::parse(info).map_err(
                        |e| PE::Error(F::Infos, E::Info(i, e)),
                    )?)
                }
                p
            },
        })
    }
}

#[derive(Debug)]
pub enum TransactionDetailError<'a> {
    FundsType(ParseError<FundsType<'a>>),
    Text(usize, str::Utf8Error),
}
impl<'a> Parsed for TransactionDetail<'a> {
    type Raw = RawTransactionDetail<'a>;
    type Parsed = ParsedTransactionDetail<'a>;
    type Field = TransactionDetailField;
    type Err = TransactionDetailError<'a>;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::ParseError as PE;
        use self::TransactionDetailField as F;
        use self::TransactionDetailError as E;
        Ok(ParsedTransactionDetail {
            type_code: parse_strint(raw.type_code, F::TypeCode)?,
            amount: parse_optstrint(raw.amount, F::Amount)?,
            funds_type: raw.funds_type.as_ref().map_or(Ok(None), |ft| {
                FundsType::parse(ft)
                    .map_err(|e| PE::Error(F::FundsType, E::FundsType(e)))
                    .map(Some)
            })?,
            bank_ref_num: parse_optstr(raw.bank_ref_num, F::BankRefNum)?,
            customer_ref_num: parse_optstr(raw.customer_ref_num, F::CustomerRefNum)?,
            text: {
                if let Some((first_char, ref raw_text)) = raw.text {
                    let mut out = Vec::with_capacity(raw_text.len());
                    let mut raw_lines = raw_text.iter().enumerate();
                    let first_raw_line = raw_lines.next().map(|x| x.1);
                    let mut first_line =
                        Vec::with_capacity(first_raw_line.map(|rl| rl.len()).unwrap_or(1));
                    first_line.push(first_char);
                    if let Some(first_raw_line) = first_raw_line {
                        first_line.extend_from_slice(first_raw_line)
                    }
                    let first_line = str::from_utf8(&*first_line).map(str::to_owned).map_err(
                        |e| {
                            PE::Error(F::Text, E::Text(0, e))
                        },
                    )?;
                    for (i, raw_line) in raw_lines {
                        out.push(str::from_utf8(raw_line).map_err(|e| {
                            PE::Error(F::Text, E::Text(i, e))
                        })?);
                    }
                    Some((first_line, out))
                } else {
                    None
                }
            },
        })
    }
}

impl<'a> Parsed for AccountTrailer<'a> {
    type Raw = RawAccountTrailer<'a>;
    type Parsed = ParsedAccountTrailer;
    type Field = AccountTrailerField;
    type Err = void::Void;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::AccountTrailerField as F;
        Ok(ParsedAccountTrailer {
            control_total: parse_strint(raw.control_total, F::ControlTotal)?,
            records_num: parse_strint(raw.records_num, F::RecordsNum)?,
        })
    }
}

impl<'a> Parsed for GroupTrailer<'a> {
    type Raw = RawGroupTrailer<'a>;
    type Parsed = ParsedGroupTrailer;
    type Field = GroupTrailerField;
    type Err = void::Void;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::GroupTrailerField as F;
        Ok(ParsedGroupTrailer {
            control_total: parse_strint(raw.control_total, F::ControlTotal)?,
            accounts_num: parse_strint(raw.accounts_num, F::AccountsNum)?,
            records_num: parse_strint(raw.records_num, F::RecordsNum)?,
        })
    }
}

impl<'a> Parsed for FileTrailer<'a> {
    type Raw = RawFileTrailer<'a>;
    type Parsed = ParsedFileTrailer;
    type Field = FileTrailerField;
    type Err = void::Void;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::FileTrailerField as F;
        Ok(ParsedFileTrailer {
            control_total: parse_strint(raw.control_total, F::ControlTotal)?,
            groups_num: parse_strint(raw.groups_num, F::GroupsNum)?,
            records_num: parse_strint(raw.records_num, F::RecordsNum)?,
        })
    }
}

#[derive(Debug)]
pub enum DateError {
    All,
    Year,
    Month,
    Day,
}
named!(
    date<&[u8], (&[u8], &[u8], &[u8]), DateError>,
    do_parse!(
        year: return_error!(::nom::ErrorKind::Custom(DateError::Year), take!(2)) >>
        month: return_error!(::nom::ErrorKind::Custom(DateError::Month), take!(2)) >>
        day: return_error!(::nom::ErrorKind::Custom(DateError::Day), take!(2)) >>
        (year, month, day)
    )
);
impl str::FromStr for Date {
    type Err = DateError;
    fn from_str(i: &str) -> Result<Date, DateError> {
        use self::DateError as E;
        let (year, month, day) = date(i.as_bytes()).to_full_result().map_err(|e| match e {
            ::nom::IError::Error(::nom::ErrorKind::Custom(err)) => err,
            _ => DateError::All,
        })?;
        let year = str::from_utf8(year).or(Err(E::Year))?;
        let month = str::from_utf8(month).or(Err(E::Month))?;
        let day = str::from_utf8(day).or(Err(E::Day))?;
        Ok(Date {
            year: year.parse().or(Err(E::Year))?,
            month: month.parse().or(Err(E::Month))?,
            day: day.parse().or(Err(E::Day))?,
        })
    }
}
#[derive(Debug)]
pub enum TimeError {
    All,
    Hour,
    Minute,
}
named!(
    time<&[u8], (&[u8], &[u8]), TimeError>,
    do_parse!(
        hour: return_error!(::nom::ErrorKind::Custom(TimeError::Hour), take!(2)) >>
        minute: return_error!(::nom::ErrorKind::Custom(TimeError::Minute), take!(2)) >>
        (hour, minute)
    )
);
impl str::FromStr for Time {
    type Err = TimeError;
    fn from_str(i: &str) -> Result<Time, TimeError> {
        use self::TimeError as E;
        let (hour, minute) = time(i.as_bytes()).to_full_result().map_err(|e| match e {
            ::nom::IError::Error(::nom::ErrorKind::Custom(err)) => err,
            _ => TimeError::All,
        })?;
        let hour = str::from_utf8(hour).or(Err(E::Hour))?;
        let minute = str::from_utf8(minute).or(Err(E::Minute))?;
        Ok(Time {
            hour: hour.parse().or(Err(E::Hour))?,
            minute: minute.parse().or(Err(E::Minute))?,
        })
    }
}

#[derive(Debug)]
pub enum AccountInfoError<'a> {
    FundsType(ParseError<FundsType<'a>>),
}
impl<'a> Parsed for AccountInfo<'a> {
    type Raw = RawAccountInfo<'a>;
    type Parsed = ParsedAccountInfo;
    type Field = AccountInfoField;
    type Err = AccountInfoError<'a>;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::ParseError as PE;
        use self::AccountInfoField as F;
        use self::AccountInfoError as E;
        Ok(ParsedAccountInfo {
            type_code: parse_optstrint(raw.type_code, F::TypeCode)?,
            amount: parse_optstrint(raw.amount, F::Amount)?,
            item_count: parse_optstrint(raw.item_count, F::ItemCount)?,
            funds_type: raw.funds_type.as_ref().map_or(Ok(None), |ft| {
                FundsType::parse(ft)
                    .map_err(|e| PE::Error(F::FundsType, E::FundsType(e)))
                    .map(Some)
            })?,
        })
    }
}

#[derive(Debug)]
pub enum FundsTypeError<'a> {
    Date(DateError),
    Time(TimeError),
    DistributedAvailDDist(usize, ParseError<DistributedAvailDistribution<'a>>),
}
impl<'a> Parsed for FundsType<'a> {
    type Raw = RawFundsType<'a>;
    type Parsed = ParsedFundsType;
    type Field = FundsTypeField;
    type Err = FundsTypeError<'a>;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::ParseError as PE;
        use self::FundsTypeField as F;
        use self::FundsTypeError as E;

        use self::RawFundsType as R;
        use self::ParsedFundsType as P;
        Ok(match *raw {
            R::Unknown => P::Unknown,
            R::ImmediateAvail => P::ImmediateAvail,
            R::OneDayAvail => P::OneDayAvail,
            R::TwoOrMoreDaysAvail => P::TwoOrMoreDaysAvail,
            R::DistributedAvailS {
                immediate,
                one_day,
                more_than_one_day,
            } => {
                P::DistributedAvailS {
                    immediate: parse_optstrint(immediate, F::DistributedAvailSImmediate)?,
                    one_day: parse_optstrint(one_day, F::DistributedAvailSOneDay)?,
                    more_than_one_day: parse_optstrint(
                        more_than_one_day,
                        F::DistributedAvailSMoreThanOneDay,
                    )?,
                }
            }
            R::ValueDated { date, time } => {
                P::ValueDated {
                    date: parse_strfrom(date, F::ValueDatedDate, E::Date)?,
                    time: parse_optstrfrom(time, F::ValueDatedTime, E::Time)?,
                }
            }
            R::DistributedAvailD { num, ref dists } => {
                P::DistributedAvailD {
                    num: parse_strint(num, F::DistributedAvailDNum)?,
                    dists: {
                        let mut p = Vec::with_capacity(dists.len());
                        for (i, dist) in dists.iter().enumerate() {
                            p.push(DistributedAvailDistribution::parse(dist).map_err(|e| {
                                PE::Error(F::DistributedAvailDDists, E::DistributedAvailDDist(i, e))
                            })?)
                        }
                        p
                    },
                }
            }
        })
    }
}

impl<'a> Parsed for DistributedAvailDistribution<'a> {
    type Raw = RawDistributedAvailDistribution<'a>;
    type Parsed = ParsedDistributedAvailDistribution;
    type Field = DistributedAvailDistributionField;
    type Err = void::Void;

    fn parse(raw: &Self::Raw) -> Result<Self::Parsed, ParseError<Self>> {
        use self::DistributedAvailDistributionField as F;
        Ok(ParsedDistributedAvailDistribution {
            days: parse_strint(raw.days, F::Days)?,
            amount: parse_strint(raw.amount, F::Amount)?,
        })
    }
}
