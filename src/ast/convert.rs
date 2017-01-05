use super::*;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub enum ChronoError<T> {
    InvalidDate,
    InvalidTime,
    None,
    Ambiguous(T, T),
}

fn chrono_res<T>(res: chrono::LocalResult<T>) -> Result<T, ChronoError<T>> {
    match res {
        chrono::LocalResult::Single(x) => Ok(x),
        chrono::LocalResult::None => Err(ChronoError::None),
        chrono::LocalResult::Ambiguous(x, y) => Err(ChronoError::Ambiguous(x, y)),
    }
}

fn chrono_date<Tz: TimeZone>(date: &Date,
                             time_zone: &Tz)
                             -> Result<chrono::Date<Tz>, ChronoError<chrono::Date<Tz>>> {
    chrono_res(time_zone.ymd_opt(if date.year > 70 { 1900 } else { 2000 } + date.year as i32,
                                 date.month as u32,
                                 date.day as u32))
}
fn chrono_date_time<Tz>(date: &Date,
                        time: &Time,
                        time_zone: &Tz,
                        end_of_day: &NaiveTime)
                        -> Result<chrono::DateTime<Tz>, ChronoError<chrono::DateTime<Tz>>>
    where Tz: TimeZone
{
    use self::ChronoError as CE;
    chrono::NaiveDate::from_ymd_opt(if date.year > 70 { 1900 } else { 2000 } + date.year as i32,
                                    date.month as u32,
                                    date.day as u32)
        .map_or(Err(CE::InvalidDate), Ok)
        .and_then(|d| {
            match *time {
                Time { hour: 99, minute: 99 } => Ok(d.and_time(*end_of_day)),
                _ => {
                    d.and_hms_opt(time.hour as u32, time.minute as u32, 0)
                        .map_or(Err(CE::InvalidTime), Ok)
                }
            }
        })
        .and_then(|dt| chrono_res(time_zone.from_local_datetime(&dt)))
}
fn chrono_date_or_time<Tz>(date: &Date,
                           time: &Option<&Time>,
                           time_zone: &Tz,
                           end_of_day: &NaiveTime)
                           -> Result<data::DateOrTime<Tz>, ChronoError<data::DateOrTime<Tz>>>
    where Tz: TimeZone
{
    use self::ChronoError as CE;
    use data::DateOrTime as DOT;
    match *time {
        Some(time) => {
            chrono_date_time(date, time, time_zone, end_of_day)
                .map(DOT::DateTime)
                .map_err(|e| match e {
                    CE::InvalidDate => CE::InvalidDate,
                    CE::InvalidTime => CE::InvalidTime,
                    CE::None => CE::None,
                    CE::Ambiguous(x, y) => CE::Ambiguous(DOT::DateTime(x), DOT::DateTime(y)),
                })
        }
        None => {
            chrono_date(date, time_zone).map(DOT::Date).map_err(|e| match e {
                CE::InvalidDate => CE::InvalidDate,
                CE::InvalidTime => CE::InvalidTime,
                CE::None => CE::None,
                CE::Ambiguous(x, y) => CE::Ambiguous(DOT::Date(x), DOT::Date(y)),
            })
        }
    }
}

#[derive(Debug)]
pub enum FileConvError<Tz: TimeZone> {
    NotFileHeader,
    Creation(ChronoError<chrono::DateTime<Tz>>),
    Group(usize, GroupConvError<Tz>),
    NotFileTrailer,
    ControlTotal(i64),
    GroupsNum(usize),
    RecordsNum(usize),
}

pub fn convert<'cur, I, Tz>(mut iter: &mut I,
                            time_zone: &Tz,
                            end_of_day: &NaiveTime)
                            -> Result<data::File<'cur, Tz>, FileConvError<Tz>>
    where I: Iterator<Item = ParsedRecord>,
          Tz: TimeZone
{
    use self::ParsedRecord as PR;
    use self::FileConvError as CE;
    match iter.next() {
        Some(PR::FileHeader(fh)) => {
            let mut control_total: i64 = 0;
            let mut groups_num: usize = 0;

            let file_trailer: ParsedFileTrailer;
            let file = data::File {
                sender: data::Party(fh.sender_ident),
                receiver: data::Party(fh.receiver_ident),
                creation: chrono_date_time(&fh.creation_date,
                                           &fh.creation_time,
                                           time_zone,
                                           end_of_day).map_err(CE::Creation)?,
                ident: data::FileIdent(fh.ident_num),
                groups: {
                    let (groups, file_t) =
                        convert_groups(iter,
                                       time_zone,
                                       end_of_day,
                                       &mut control_total,
                                       &mut groups_num).map_err(|(i, e)| CE::Group(i, e))?;
                    file_trailer = file_t;
                    groups
                },
            };
            if file_trailer.control_total != control_total {
                return Err(CE::ControlTotal(control_total));
            }
            if file_trailer.groups_num != groups_num {
                return Err(CE::GroupsNum(groups_num));
            }
            // TODO records_num
            Ok(file)
        }
        _ => Err(CE::NotFileHeader),
    }
}

#[derive(Debug)]
pub enum GroupConvError<Tz: TimeZone> {
    NotGroupHeaderOrFileTrailer,
    Status,
    AsOf(ChronoError<data::DateOrTime<Tz>>),
    Currency(String),
    AsOfDateMod,
    Account(usize, AccountConvError<Tz>),
    NotGroupTrailer,
    ControlTotal(i64),
    AccountsNum(usize),
    RecordsNum(usize),
}

fn convert_groups<'cur, I, Tz>
    (mut iter: &mut I,
     time_zone: &Tz,
     end_of_day: &NaiveTime,
     file_control_total: &mut i64,
     file_groups_num: &mut usize)
     -> Result<(Vec<data::Group<'cur, Tz>>, ParsedFileTrailer), (usize, GroupConvError<Tz>)>
    where I: Iterator<Item = ParsedRecord>,
          Tz: TimeZone
{
    use self::ParsedRecord as PR;
    use self::GroupConvError as CE;

    let mut groups = Vec::new();
    let file_trailer: ParsedFileTrailer;
    loop {
        match iter.next() {
            Some(PR::GroupHeader(gh)) => {
                let mut group_control_total: i64 = 0;
                let mut group_accounts_num: usize = 0;

                let group_trailer: ParsedGroupTrailer;
                let group = data::Group {
                    ultimate_receiver: gh.ultimate_receiver_ident.map(data::Party),
                    originator: gh.originator_ident.map(data::Party),
                    status: gh.status
                        .try_into()
                        .map_err(|_| (*file_groups_num, CE::Status))?,
                    as_of: {
                        chrono_date_or_time(&gh.as_of_date,
                                          &gh.as_of_time.as_ref(),
                                          time_zone,
                                          end_of_day).map_err(|e| (*file_groups_num, CE::AsOf(e)))?
                    },
                    currency: gh.currency
                        .map_or(Ok(None), |s| {
                            penny::CURRENCIES.get(&*s)
                                .ok_or((*file_groups_num, CE::Currency(s)))
                                .map(Some)
                        })?,
                    as_of_date_mod: gh.as_of_date_mod
                        .map_or(Ok(None), |m| {
                            m.try_into()
                                .map_err(|_| (*file_groups_num, CE::AsOfDateMod))
                                .map(Some)
                        })?,
                    accounts: {
                        let (accounts, group_t) = convert_accounts(
                                iter,
                                time_zone,
                                end_of_day,
                                &mut group_control_total,
                                &mut group_accounts_num
                            ).map_err(|(i, e)| (*file_groups_num, CE::Account(i, e)))?;
                        group_trailer = group_t;
                        accounts
                    },
                };
                if group_trailer.control_total != group_control_total {
                    return Err((*file_groups_num, CE::ControlTotal(group_control_total)));
                }
                if group_trailer.accounts_num != group_accounts_num {
                    return Err((*file_groups_num, CE::AccountsNum(group_accounts_num)));
                }
                // TODO records_num
                groups.push(group);
                *file_control_total += group_control_total;
                *file_groups_num += 1;
            }
            Some(PR::FileTrailer(ft)) => {
                file_trailer = ft;
                break;
            }
            _ => return Err((*file_groups_num, CE::NotGroupHeaderOrFileTrailer)),
        }
    }
    Ok((groups, file_trailer))
}

#[derive(Debug)]
pub enum AccountConvError<Tz: TimeZone> {
    NotTransactionDetailOrAccountTrailer,
    NotAccountTrailer,
    NotAccountIdentOrGroupTrailer,
    Currency,
    AccountInfo(usize, AccountInfoConvError<Tz>),
    TransactionDetail(usize, TransactionDetailConvError<Tz>),
    ControlTotal(i64),
    RecordsNum(usize),
}

fn convert_accounts<'cur, I, Tz>
    (mut iter: &mut I,
     time_zone: &Tz,
     end_of_day: &NaiveTime,
     group_control_total: &mut i64,
     group_accounts_num: &mut usize)
     -> Result<(Vec<data::Account<'cur, Tz>>, ParsedGroupTrailer), (usize, AccountConvError<Tz>)>
    where I: Iterator<Item = ParsedRecord>,
          Tz: TimeZone
{
    use self::ParsedRecord as PR;
    use self::AccountConvError as CE;

    let mut accounts = Vec::new();
    let group_trailer: ParsedGroupTrailer;
    loop {
        match iter.next() {
            Some(PR::AccountIdent(ah)) => {
                let mut account_control_total: i64 = 0;

                let account_trailer: ParsedAccountTrailer;
                let account = data::Account {
                    customer_account: data::AccountNumber(ah.customer_account_num),
                    currency: ah.currency
                        .map_or(Ok(None), |s| {
                            penny::CURRENCIES.get(&*s)
                                .ok_or((*group_accounts_num, CE::Currency))
                                .map(Some)
                        })?,
                    infos: {
                        let (infos, control_total) = convert_infos(&ah.infos, time_zone, end_of_day)
                            .map_err(|(i, e)| (*group_accounts_num, CE::AccountInfo(i, e)))?;
                        account_control_total += control_total;
                        infos
                    },
                    transaction_details: {
                        let mut transaction_num: usize = 0;
                        let (transactions, account_t) = convert_transaction_details(
                            iter,
                            time_zone,
                            end_of_day,
                            &mut account_control_total,
                            &mut transaction_num,
                        ).map_err(|e| (*group_accounts_num, CE::TransactionDetail(transaction_num, e)))?;
                        account_trailer = account_t;
                        transactions
                    },
                };
                if account_trailer.control_total != account_control_total {
                    return Err((*group_accounts_num, CE::ControlTotal(account_control_total)));
                }
                // TODO records_num
                accounts.push(account);
                *group_control_total += account_control_total;
                *group_accounts_num += 1;
            }
            Some(PR::GroupTrailer(gt)) => {
                group_trailer = gt;
                break;
            }
            _ => return Err((*group_accounts_num, CE::NotAccountIdentOrGroupTrailer)),
        }
    }
    Ok((accounts, group_trailer))
}

#[derive(Debug)]
pub enum AccountInfoConvError<Tz: TimeZone> {
    NoCode,
    InvalidCode,
    StatusItemCount,
    StatusFunds,
    SummaryNegativeAmount,
    Funds(FundsTypeConvError<Tz>),
}

fn convert_infos<Tz>
    (pinfos: &[ParsedAccountInfo],
     time_zone: &Tz,
     end_of_day: &NaiveTime)
     -> Result<(Vec<data::AccountInfo<Tz>>, i64), (usize, AccountInfoConvError<Tz>)>
    where Tz: TimeZone
{
    use data::AccountInfo as AI;
    use self::AccountInfoConvError as CE;

    let mut control_total = 0;
    let mut infos = Vec::with_capacity(pinfos.len());
    for (i, pi) in pinfos.iter().enumerate() {
        match (pi.type_code, pi.amount, pi.item_count, &pi.funds_type) {
            (None, None, None, &None) => (),
            (Some(code), amount, item_count, funds) => {
                if let Ok(code) = data::StatusCode::try_from(code) {
                    match (item_count, funds) {
                        (None, &None) => {
                            infos.push(AI::Status {
                                code: code,
                                amount: {
                                    if let Some(a) = amount {
                                        control_total += a;
                                    }
                                    amount
                                },
                            })
                        }
                        (Some(_), _) => return Err((i, CE::StatusItemCount)),
                        (_, &Some(_)) => return Err((i, CE::StatusFunds)),
                    }
                } else if let Ok(code) = data::SummaryCode::try_from(code) {
                    infos.push(AI::Summary {
                        code: code,
                        amount: amount.map_or(Ok(None), |a| {
                                if a >= 0 {
                                    control_total += a;
                                    Ok(Some(a as u64))
                                } else {
                                    Err((i, CE::SummaryNegativeAmount))
                                }
                            })?,
                        item_count: item_count,
                        funds: funds.as_ref()
                            .map_or(Ok(None),
                                    |f| convert_funds_type(f, time_zone, end_of_day).map(Some))
                            .map_err(|e| (i, CE::Funds(e)))?,
                    })
                } else {
                    return Err((i, CE::InvalidCode));
                }
            }
            _ => return Err((i, CE::NoCode)),
        };
    }
    Ok((infos, control_total))
}

#[derive(Debug)]
pub enum FundsTypeConvError<Tz: TimeZone> {
    ValueDated(ChronoError<data::DateOrTime<Tz>>),
    DistributedAvailDNum(usize),
}

fn convert_funds_type<Tz>(funds_type: &ParsedFundsType,
                          time_zone: &Tz,
                          end_of_day: &NaiveTime)
                          -> Result<data::FundsType<Tz>, FundsTypeConvError<Tz>>
    where Tz: TimeZone
{
    use self::ParsedFundsType as PFT;
    use data::FundsType as FT;
    use self::FundsTypeConvError as CE;

    Ok(match *funds_type {
        PFT::Unknown => FT::Unknown,
        PFT::ImmediateAvail => FT::ImmediateAvail,
        PFT::OneDayAvail => FT::OneDayAvail,
        PFT::TwoOrMoreDaysAvail => FT::TwoOrMoreDaysAvail,
        PFT::DistributedAvailS { immediate, one_day, more_than_one_day } => {
            FT::DistributedAvailS {
                immediate: immediate,
                one_day: one_day,
                more_than_one_day: more_than_one_day,
            }
        }
        PFT::ValueDated { ref date, ref time } => {
            chrono_date_or_time(date, &time.as_ref(), time_zone, end_of_day).map_err(CE::ValueDated)
                .map(FT::ValueDated)?
        }
        PFT::DistributedAvailD { num, ref dists } => {
            let ndists = dists.len();
            if num != ndists {
                return Err(CE::DistributedAvailDNum(ndists));
            }
            FT::DistributedAvailD(dists.iter()
                .map(|&ParsedDistributedAvailDistribution { days, amount }| {
                    data::DistributedAvailDistribution {
                        days: days,
                        amount: amount,
                    }
                })
                .collect())
        }
    })
}

#[derive(Debug)]
pub enum TransactionDetailConvError<Tz: TimeZone> {
    NotTransactionDetailOrAccountTrailer,
    DetailCode(u16),
    Funds(FundsTypeConvError<Tz>),
}

fn convert_transaction_details<I, Tz>
    (iter: &mut I,
     time_zone: &Tz,
     end_of_day: &NaiveTime,
     account_control_total: &mut i64,
     transaction_num: &mut usize)
     -> Result<(Vec<data::TransactionDetail<Tz>>, ParsedAccountTrailer), TransactionDetailConvError<Tz>>
    where I: Iterator<Item = ParsedRecord>,
          Tz: TimeZone
{
    use self::ParsedRecord as PR;
    use self::TransactionDetailConvError as CE;

    let mut transactions = Vec::new();
    let account_trailer: ParsedAccountTrailer;
    loop {
        match iter.next() {
            Some(PR::TransactionDetail(td)) => {
                transactions.push(data::TransactionDetail {
                    code: data::DetailCode::try_from(td.type_code).map_err(CE::DetailCode)?,
                    amount: {
                        if let Some(a) = td.amount {
                            *account_control_total += a as i64;
                        }
                        td.amount
                    },
                    funds: td.funds_type
                        .as_ref()
                        .map_or(Ok(None),
                                |ft| convert_funds_type(ft, time_zone, end_of_day).map(Some))
                        .map_err(CE::Funds)?,
                    bank_ref_num: td.bank_ref_num.map(data::ReferenceNum),
                    customer_ref_num: td.customer_ref_num.map(data::ReferenceNum),
                    text: td.text,
                });
                *transaction_num += 1;
            }
            Some(PR::AccountTrailer(at)) => {
                account_trailer = at;
                break;
            }
            _ => return Err(CE::NotTransactionDetailOrAccountTrailer),
        }
    }
    Ok((transactions, account_trailer))
}
