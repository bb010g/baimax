use std::convert::{TryFrom, TryInto};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use penny;

use ast;
use ast::data;
use ast::data::NaiveDateOrTime;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum ChronoError {
    InvalidDate,
    InvalidTime,
}

fn chrono_date(date: &ast::Date) -> Result<NaiveDate, ChronoError> {
    NaiveDate::from_ymd_opt(
        if date.year > 70 { 1900 } else { 2000 } + date.year as i32,
        date.month as u32,
        date.day as u32,
    ).ok_or(ChronoError::InvalidDate)
}
fn chrono_date_time(
    date: &ast::Date,
    time: &ast::Time,
    end_of_day: &NaiveTime,
) -> Result<NaiveDateTime, ChronoError> {
    chrono_date(date).and_then(|d| match *time {
        ast::Time {
            hour: 99,
            minute: 99,
        } => Ok(d.and_time(*end_of_day)),
        _ => {
            d.and_hms_opt(time.hour as u32, time.minute as u32, 0)
                .map_or(Err(ChronoError::InvalidTime), Ok)
        }
    })
}
fn chrono_date_or_time(
    date: &ast::Date,
    time: Option<&ast::Time>,
    end_of_day: &NaiveTime,
) -> Result<NaiveDateOrTime, ChronoError> {
    use ast::data::NaiveDateOrTime as NDOT;
    match time {
        Some(time) => chrono_date_time(date, time, end_of_day).map(NDOT::DateTime),
        None => chrono_date(date).map(NDOT::Date),
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum FileConvError {
    NotFileHeader,
    Creation(ChronoError),
    Group(usize, GroupConvError),
    NotFileTrailer,
    ControlTotal(i64),
    GroupsNum(usize),
    RecordsNum(usize),
}

pub fn convert<'a, I>(mut iter: &mut I, end_of_day: &NaiveTime) -> Result<data::File, FileConvError>
where
    I: Iterator<Item = ast::ParsedRecord<'a>>,
{
    use ast::ParsedRecord as PR;
    use self::FileConvError as CE;
    match iter.next() {
        Some(PR::FileHeader(fh)) => {
            let mut control_total: i64 = 0;
            let mut groups_num: usize = 0;

            let file_trailer: ast::ParsedFileTrailer;
            let file = data::File {
                sender: data::Party(fh.sender_ident.to_owned()),
                receiver: data::Party(fh.receiver_ident.to_owned()),
                creation: chrono_date_time(&fh.creation_date, &fh.creation_time, end_of_day)
                    .map_err(CE::Creation)?,
                ident: data::FileIdent(fh.ident_num),
                groups: {
                    let (groups, file_t) =
                        convert_groups(iter, end_of_day, &mut control_total, &mut groups_num)
                            .map_err(|(i, e)| CE::Group(i, e))?;
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum GroupConvError {
    NotGroupHeaderOrFileTrailer,
    Status,
    AsOf(ChronoError),
    Currency(String),
    AsOfDateMod,
    Account(usize, AccountConvError),
    NotGroupTrailer,
    ControlTotal(i64),
    AccountsNum(usize),
    RecordsNum(usize),
}

fn convert_groups<'a, I>(
    mut iter: &mut I,
    end_of_day: &NaiveTime,
    file_control_total: &mut i64,
    file_groups_num: &mut usize,
) -> Result<(Vec<data::Group>, ast::ParsedFileTrailer), (usize, GroupConvError)>
where
    I: Iterator<Item = ast::ParsedRecord<'a>>,
{
    use ast::ParsedRecord as PR;
    use self::GroupConvError as CE;

    let mut groups = Vec::new();
    let file_trailer: ast::ParsedFileTrailer;
    loop {
        match iter.next() {
            Some(PR::GroupHeader(gh)) => {
                let mut group_control_total: i64 = 0;
                let mut group_accounts_num: usize = 0;

                let group_trailer: ast::ParsedGroupTrailer;
                let group = data::Group {
                    ultimate_receiver: gh.ultimate_receiver_ident.map(
                        |s| data::Party(s.to_owned()),
                    ),
                    originator: gh.originator_ident.map(|s| data::Party(s.to_owned())),
                    status: gh.status.try_into().map_err(
                        |_| (*file_groups_num, CE::Status),
                    )?,
                    as_of: {
                        chrono_date_or_time(&gh.as_of_date, gh.as_of_time.as_ref(), end_of_day)
                            .map_err(|e| (*file_groups_num, CE::AsOf(e)))?
                    },
                    currency: gh.currency.map_or(Ok(None), |s| {
                        s.parse::<penny::Currency>().map(Some).map_err(|_| {
                            (*file_groups_num, CE::Currency(s.to_owned()))
                        })
                    })?,
                    as_of_date_mod: gh.as_of_date_mod.map_or(Ok(None), |m| {
                        m.try_into()
                            .map_err(|_| (*file_groups_num, CE::AsOfDateMod))
                            .map(Some)
                    })?,
                    accounts: {
                        let (accounts, group_t) =
                            convert_accounts(
                                iter,
                                end_of_day,
                                &mut group_control_total,
                                &mut group_accounts_num,
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum AccountConvError {
    NotTransactionDetailOrAccountTrailer,
    NotAccountTrailer,
    NotAccountIdentOrGroupTrailer,
    Currency(String),
    AccountInfo(usize, AccountInfoConvError),
    TransactionDetail(usize, TransactionDetailConvError),
    ControlTotal(i64),
    RecordsNum(usize),
}

fn convert_accounts<'a, I>(
    mut iter: &mut I,
    end_of_day: &NaiveTime,
    group_control_total: &mut i64,
    group_accounts_num: &mut usize,
) -> Result<(Vec<data::Account>, ast::ParsedGroupTrailer), (usize, AccountConvError)>
where
    I: Iterator<Item = ast::ParsedRecord<'a>>,
{
    use ast::ParsedRecord as PR;
    use self::AccountConvError as CE;

    let mut accounts = Vec::new();
    let group_trailer: ast::ParsedGroupTrailer;
    loop {
        match iter.next() {
            Some(PR::AccountIdent(ah)) => {
                let mut account_control_total: i64 = 0;

                let account_trailer: ast::ParsedAccountTrailer;
                let account = data::Account {
                    customer_account: data::AccountNumber(ah.customer_account_num.to_owned()),
                    currency: ah.currency.map_or(Ok(None), |s| {
                        s.parse::<penny::Currency>().map(Some).map_err(|_| {
                            (*group_accounts_num, CE::Currency(s.to_owned()))
                        })
                    })?,
                    infos: {
                        let (infos, control_total) =
                            convert_infos(&ah.infos, end_of_day).map_err(|(i, e)| {
                                (*group_accounts_num, CE::AccountInfo(i, e))
                            })?;
                        account_control_total += control_total;
                        infos
                    },
                    transaction_details: {
                        let mut transaction_num: usize = 0;
                        let (transactions, account_t) = convert_transaction_details(
                            iter,
                            end_of_day,
                            &mut account_control_total,
                            &mut transaction_num,
                        ).map_err(|e| {
                            (
                                *group_accounts_num,
                                CE::TransactionDetail(transaction_num, e),
                            )
                        })?;
                        account_trailer = account_t;
                        transactions
                    },
                };
                if account_trailer.control_total != account_control_total {
                    return Err((
                        *group_accounts_num,
                        CE::ControlTotal(account_control_total),
                    ));
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

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum AccountInfoConvError {
    NoCode,
    InvalidCode,
    StatusItemCount,
    StatusFunds,
    SummaryNegativeAmount,
    Funds(FundsTypeConvError),
}

fn convert_infos(
    pinfos: &[ast::ParsedAccountInfo],
    end_of_day: &NaiveTime,
) -> Result<(Vec<data::AccountInfo>, i64), (usize, AccountInfoConvError)> {
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
                        amount: amount.map_or(Ok(None), |a| if a >= 0 {
                            control_total += a;
                            Ok(Some(a as u64))
                        } else {
                            Err((i, CE::SummaryNegativeAmount))
                        })?,
                        item_count: item_count,
                        funds: funds
                            .as_ref()
                            .map_or(Ok(None), |f| convert_funds_type(f, end_of_day).map(Some))
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

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum FundsTypeConvError {
    ValueDated(ChronoError),
    DistributedAvailDNum(usize),
}

fn convert_funds_type(
    funds_type: &ast::ParsedFundsType,
    end_of_day: &NaiveTime,
) -> Result<data::FundsType, FundsTypeConvError> {
    use ast::ParsedFundsType as PFT;
    use ast::data::FundsType as FT;
    use self::FundsTypeConvError as CE;

    Ok(match *funds_type {
        PFT::Unknown => FT::Unknown,
        PFT::ImmediateAvail => FT::ImmediateAvail,
        PFT::OneDayAvail => FT::OneDayAvail,
        PFT::TwoOrMoreDaysAvail => FT::TwoOrMoreDaysAvail,
        PFT::DistributedAvailS {
            immediate,
            one_day,
            more_than_one_day,
        } => {
            FT::DistributedAvailS {
                immediate,
                one_day,
                more_than_one_day,
            }
        }
        PFT::ValueDated { ref date, ref time } => {
            chrono_date_or_time(date, time.as_ref(), end_of_day)
                .map_err(CE::ValueDated)
                .map(FT::ValueDated)?
        }
        PFT::DistributedAvailD { num, ref dists } => {
            let ndists = dists.len();
            if num != ndists {
                return Err(CE::DistributedAvailDNum(ndists));
            }
            FT::DistributedAvailD(
                dists
                    .iter()
                    .map(|&ast::ParsedDistributedAvailDistribution {
                         days,
                         amount,
                     }| {
                        data::DistributedAvailDistribution { days, amount }
                    })
                    .collect(),
            )
        }
    })
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum TransactionDetailConvError {
    NotTransactionDetailOrAccountTrailer,
    DetailCode(u16),
    Funds(FundsTypeConvError),
}

fn convert_transaction_details<'a, I>(
    iter: &mut I,
    end_of_day: &NaiveTime,
    account_control_total: &mut i64,
    transaction_num: &mut usize,
) -> Result<(Vec<data::TransactionDetail>, ast::ParsedAccountTrailer), TransactionDetailConvError>
where
    I: Iterator<Item = ast::ParsedRecord<'a>>,
{
    use ast::ParsedRecord as PR;
    use self::TransactionDetailConvError as CE;

    let mut transactions = Vec::new();
    let account_trailer: ast::ParsedAccountTrailer;
    loop {
        match iter.next() {
            Some(PR::TransactionDetail(td)) => {
                transactions.push(data::TransactionDetail {
                    code: data::DetailCode::try_from(td.type_code).map_err(
                        CE::DetailCode,
                    )?,
                    amount: {
                        if let Some(a) = td.amount {
                            *account_control_total += a as i64;
                        }
                        td.amount
                    },
                    funds: td.funds_type
                        .as_ref()
                        .map_or(Ok(None), |ft| convert_funds_type(ft, end_of_day).map(Some))
                        .map_err(CE::Funds)?,
                    bank_ref_num: td.bank_ref_num.map(|s| data::ReferenceNum(s.to_owned())),
                    customer_ref_num: td.customer_ref_num.map(
                        |s| data::ReferenceNum(s.to_owned()),
                    ),
                    text: td.text.map(|v| {
                        ::std::iter::once(v.0)
                            .chain(v.1.into_iter().map(str::to_owned))
                            .collect::<Vec<_>>()
                    }),
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
