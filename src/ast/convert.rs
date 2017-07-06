use std::convert::{TryFrom, TryInto};

use chrono::NaiveDate;
use itertools::Itertools;
use penny;

use ast::{self, ParsedRecord};
use ast::data::{self, BaiDateOrTime, BaiDateTime};

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
fn chrono_date_time(date: &ast::Date, time: &ast::Time) -> Result<BaiDateTime, ChronoError> {
    chrono_date(date).and_then(|date| match *time {
        ast::Time {
            hour: 99,
            minute: 99,
        } => Ok(BaiDateTime::DateEndOfDay(date)),
        _ => {
            date.and_hms_opt(time.hour as u32, time.minute as u32, 0)
                .map(BaiDateTime::DateTime)
                .map_or(Err(ChronoError::InvalidTime), Ok)
        }
    })
}
fn chrono_date_or_time(
    date: &ast::Date,
    time: Option<&ast::Time>,
) -> Result<BaiDateOrTime, ChronoError> {
    use ast::data::BaiDateOrTime as BDOT;
    match time {
        Some(time) => chrono_date_time(date, time).map(BDOT::from),
        None => chrono_date(date).map(BDOT::from),
    }
}

pub struct Converter {
    state: Option<ConverterState>,
}

impl Default for Converter {
    fn default() -> Self {
        Converter {
            state: Some(ConverterState::Fresh),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
#[cfg_attr(any(feature = "clippy", feature = "cargo-clippy"), allow(large_enum_variant))]
enum ConverterState {
    Fresh,
    File(FileConvState),
    Group(FileConvState, GroupConvState),
    Account(FileConvState, GroupConvState, AccountConvState),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
struct FileConvState {
    data: data::File,
    records_num: usize,
    control_total: i64,
}
impl FileConvState {
    fn new(data: data::File, records_num: usize) -> Self {
        FileConvState {
            data,
            records_num,
            control_total: 0,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
struct GroupConvState {
    data: data::Group,
    records_num: usize,
    control_total: i64,
}
impl GroupConvState {
    fn new(data: data::Group, records_num: usize) -> Self {
        GroupConvState {
            data,
            records_num,
            control_total: 0,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
struct AccountConvState {
    data: data::Account,
    records_num: usize,
    control_total: i64,
}

impl Default for ConverterState {
    fn default() -> Self {
        ConverterState::Fresh
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum ConverterProgress {
    Fresh,
    File,
    Group,
    Account,
}

impl ConverterState {
    pub fn progress(&self) -> ConverterProgress {
        match *self {
            ConverterState::Fresh => ConverterProgress::Fresh,
            ConverterState::File { .. } => ConverterProgress::File,
            ConverterState::Group { .. } => ConverterProgress::Group,
            ConverterState::Account { .. } => ConverterProgress::Account,
        }
    }

    fn unwrap_file(&self) -> &FileConvState {
        match *self {
            ConverterState::File(ref f) => f,
            ref s => panic!("ConverterState::{:?} is not File", s.progress()),
        }
    }
    fn unwrap_file_move(self) -> FileConvState {
        match self {
            ConverterState::File(f) => f,
            s => panic!("ConverterState::{:?} is not File", s.progress()),
        }
    }
    fn unwrap_group(&self) -> (&FileConvState, &GroupConvState) {
        match *self {
            ConverterState::Group(ref f, ref g) => (f, g),
            ref s => panic!("ConverterState::{:?} is not Group", s.progress()),
        }
    }
    fn unwrap_group_move(self) -> (FileConvState, GroupConvState) {
        match self {
            ConverterState::Group(f, g) => (f, g),
            s => panic!("ConverterState::{:?} is not Group", s.progress()),
        }
    }
    fn unwrap_account(&self) -> (&FileConvState, &GroupConvState, &AccountConvState) {
        match *self {
            ConverterState::Account(ref f, ref g, ref a) => (f, g, a),
            ref s => panic!("ConverterState::{:?} is not Account", s.progress()),
        }
    }
    fn unwrap_account_mut(
        &mut self,
    ) -> (
        &mut FileConvState,
        &mut GroupConvState,
        &mut AccountConvState,
    ) {
        match *self {
            ConverterState::Account(ref mut f, ref mut g, ref mut a) => (f, g, a),
            ref s => panic!("ConverterState::{:?} is not Account", s.progress()),
        }
    }
    fn unwrap_account_move(self) -> (FileConvState, GroupConvState, AccountConvState) {
        match self {
            ConverterState::Account(f, g, a) => (f, g, a),
            s => panic!("ConverterState::{:?} is not Account", s.progress()),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum ConvertError {
    RecordType {
        record: usize,
        progress: ConverterProgress,
    },
    File(FileConvError),
    Group { group: usize, err: GroupConvError },
    Account {
        group: usize,
        account: usize,
        err: AccountConvError,
    },
    TransactionDetail {
        group: usize,
        account: usize,
        transaction: usize,
        err: TransactionDetailConvError,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum ConverterOutput {
    Active,
    Ok(data::File),
    Err(ConvertError),
    Done,
}
impl From<Option<Result<Option<data::File>, ConvertError>>> for ConverterOutput {
    fn from(file: Option<Result<Option<data::File>, ConvertError>>) -> Self {
        match file {
            Some(Ok(None)) => ConverterOutput::Active,
            Some(Ok(Some(file))) => ConverterOutput::Ok(file),
            Some(Err(e)) => ConverterOutput::Err(e),
            None => ConverterOutput::Done,
        }
    }
}
impl ConverterOutput {
    pub fn expand(self) -> Option<Result<Option<data::File>, ConvertError>> {
        match self {
            ConverterOutput::Active => Some(Ok(None)),
            ConverterOutput::Ok(file) => Some(Ok(Some(file))),
            ConverterOutput::Err(e) => Some(Err(e)),
            ConverterOutput::Done => None,
        }
    }
}

impl Converter {
    pub fn process<'a>(&mut self, record: ParsedRecord<'a>) -> ConverterOutput {
        let progress = match self.state {
            Some(ref state) => state.progress(),
            None => return ConverterOutput::Done,
        };
        match progress {
            ConverterProgress::Fresh => {
                match record {
                    ParsedRecord::FileHeader(fh) => {
                        match fh.convert() {
                            Ok(file) => {
                                self.state =
                                    Some(ConverterState::File(FileConvState::new(file, 1)));
                                ConverterOutput::Active
                            }
                            Err(e) => ConverterOutput::Err(ConvertError::File(e)),
                        }
                    }
                    _ => {
                        self.state = None;
                        ConverterOutput::Err(ConvertError::RecordType {
                            record: 0,
                            progress,
                        })
                    }
                }
            }
            ConverterProgress::File => {
                match record {
                    ParsedRecord::GroupHeader(gh) => {
                        match gh.convert() {
                            Ok(group) => {
                                let file = self.state.take().unwrap().unwrap_file_move();
                                self.state = Some(
                                    ConverterState::Group(file, GroupConvState::new(group, 1)),
                                );
                                ConverterOutput::Active
                            }
                            Err(err) => {
                                let group_num =
                                    self.state.as_ref().unwrap().unwrap_file().data.groups.len();
                                self.state = None;
                                ConverterOutput::Err(ConvertError::Group {
                                    group: group_num,
                                    err,
                                })
                            }
                        }
                    }
                    ParsedRecord::FileTrailer(ft) => {
                        let (control_total, groups_num) = {
                            let file = self.state.as_ref().unwrap().unwrap_file();
                            (file.control_total, file.data.groups.len())
                        };
                        // TODO verify records_num
                        if ft.control_total != control_total {
                            self.state = None;
                            ConverterOutput::Err(ConvertError::File(FileConvError::ControlTotal {
                                expected: ft.control_total,
                                actual: control_total,
                            }))
                        } else if ft.groups_num != groups_num {
                            self.state = None;
                            ConverterOutput::Err(ConvertError::File(FileConvError::GroupsNum {
                                expected: ft.groups_num,
                                actual: groups_num,
                            }))
                        } else {
                            let file = self.state.take().unwrap().unwrap_file_move();
                            ConverterOutput::Ok(file.data)
                        }
                    }
                    _ => {
                        let record = self.state.as_ref().unwrap().unwrap_file().records_num;
                        self.state = None;
                        ConverterOutput::Err(ConvertError::RecordType { record, progress })
                    }
                }
            }
            ConverterProgress::Group => {
                match record {
                    ParsedRecord::AccountIdent(ai) => {
                        match ai.convert() {
                            Ok((account, control_total)) => {
                                let (file, group) = self.state.take().unwrap().unwrap_group_move();
                                self.state = Some(ConverterState::Account(
                                    file,
                                    group,
                                    AccountConvState {
                                        data: account,
                                        records_num: 1,
                                        control_total,
                                    },
                                ));
                                ConverterOutput::Active
                            }
                            Err(err) => {
                                let (group_num, account_num) = {
                                    let (file, group) = self.state.as_ref().unwrap().unwrap_group();
                                    (file.data.groups.len(), group.data.accounts.len())
                                };
                                ConverterOutput::Err(ConvertError::Account {
                                    group: group_num,
                                    account: account_num,
                                    err,
                                })
                            }
                        }
                    }
                    ParsedRecord::GroupTrailer(gt) => {
                        let (group, control_total, accounts_num) = {
                            let (file, group) = self.state.as_ref().unwrap().unwrap_group();
                            (
                                file.data.groups.len(),
                                group.control_total,
                                group.data.accounts.len(),
                            )
                        };
                        // TODO verify records_num
                        if gt.control_total != control_total {
                            self.state = None;
                            ConverterOutput::Err(ConvertError::Group {
                                group,
                                err: GroupConvError::ControlTotal {
                                    expected: gt.control_total,
                                    actual: control_total,
                                },
                            })
                        } else if gt.accounts_num != accounts_num {
                            self.state = None;
                            ConverterOutput::Err(ConvertError::Group {
                                group,
                                err: GroupConvError::AccountsNum {
                                    expected: gt.accounts_num,
                                    actual: accounts_num,
                                },
                            })
                        } else {
                            let (mut file, group) = self.state.take().unwrap().unwrap_group_move();
                            file.data.groups.push(group.data);
                            file.records_num += group.records_num + 1;
                            file.control_total += group.control_total;
                            self.state = Some(ConverterState::File(file));
                            ConverterOutput::Active
                        }
                    }
                    _ => {
                        let record = self.state.as_ref().unwrap().unwrap_group().0.records_num;
                        self.state = None;
                        ConverterOutput::Err(ConvertError::RecordType { record, progress })
                    }
                }
            }
            ConverterProgress::Account => {
                match record {
                    ParsedRecord::TransactionDetail(td) => {
                        match td.convert() {
                            Ok((transaction_detail, control_total)) => {
                                let (_file, _group, account) =
                                    self.state.as_mut().unwrap().unwrap_account_mut();
                                account.data.transaction_details.push(transaction_detail);
                                account.records_num += 1;
                                account.control_total += control_total;
                                ConverterOutput::Active
                            }
                            Err(err) => {
                                let (group_num, account_num, transaction_num) = {
                                    let (file, group, account) =
                                        self.state.as_ref().unwrap().unwrap_account();
                                    (
                                        file.data.groups.len(),
                                        group.data.accounts.len(),
                                        account.data.transaction_details.len(),
                                    )
                                };
                                self.state = None;
                                ConverterOutput::Err(ConvertError::TransactionDetail {
                                    group: group_num,
                                    account: account_num,
                                    transaction: transaction_num,
                                    err,
                                })
                            }
                        }
                    }
                    ParsedRecord::AccountTrailer(at) => {
                        let (group, account, control_total) = {
                            let (file, group, account) =
                                self.state.as_ref().unwrap().unwrap_account();
                            (
                                file.data.groups.len(),
                                group.data.accounts.len(),
                                account.control_total,
                            )
                        };
                        // TODO verify records_num
                        if at.control_total != control_total {
                            self.state = None;
                            ConverterOutput::Err(ConvertError::Account {
                                group,
                                account,
                                err: AccountConvError::ControlTotal {
                                    expected: at.control_total,
                                    actual: control_total,
                                },
                            })
                        } else {
                            let (file, mut group, account) =
                                self.state.take().unwrap().unwrap_account_move();
                            group.data.accounts.push(account.data);
                            group.records_num += account.records_num + 1;
                            group.control_total += account.control_total;
                            self.state = Some(ConverterState::Group(file, group));
                            ConverterOutput::Active
                        }
                    }
                    _ => {
                        let record = self.state.as_ref().unwrap().unwrap_account().0.records_num;
                        self.state = None;
                        ConverterOutput::Err(ConvertError::RecordType { record, progress })
                    }
                }
            }
        }
    }

    pub fn fold<'a, I>(iter: &mut I) -> Result<data::File, Option<ConvertError>>
    where
        I: Iterator<Item = ast::ParsedRecord<'a>>,
    {
        let mut converter = ast::convert::Converter::default();
        match iter.fold(
            ConverterOutput::Active,
            |acc, r| match converter.process(r) {
                ConverterOutput::Done => acc,
                o => o,
            },
        ) {
            ConverterOutput::Done => unreachable!(),
            ConverterOutput::Err(e) => Err(Some(e)),
            ConverterOutput::Ok(data) => Ok(data),
            ConverterOutput::Active => Err(None),
        }
    }

    pub fn fold_results<'a, E, I, O>(iter: &mut I, op: O) -> Result<data::File, E>
    where
        I: Iterator<Item = Result<ast::ParsedRecord<'a>, E>>,
        O: FnOnce(Option<ConvertError>) -> E,
    {
        let mut converter = ast::convert::Converter::default();
        match iter.fold_results(
            ConverterOutput::Active,
            |acc, r| match converter.process(r) {
                ConverterOutput::Done => acc,
                o => o,
            },
        ) {
            Ok(ConverterOutput::Done) => unreachable!(),
            Ok(ConverterOutput::Err(e)) => Err(op(Some(e))),
            Ok(ConverterOutput::Ok(data)) => Ok(data),
            Ok(ConverterOutput::Active) => Err(op(None)),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum FileConvError {
    Creation(ChronoError),
    ControlTotal { expected: i64, actual: i64 },
    GroupsNum { expected: usize, actual: usize },
    RecordsNum { expected: usize, actual: usize },
}

impl<'a> ast::ParsedFileHeader<'a> {
    fn convert(&self) -> Result<data::File, FileConvError> {
        Ok(data::File {
            sender: data::Party(self.sender_ident.to_owned()),
            receiver: data::Party(self.receiver_ident.to_owned()),
            creation: chrono_date_time(&self.creation_date, &self.creation_time)
                .map_err(FileConvError::Creation)?,
            ident: data::FileIdent(self.ident_num),
            groups: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum GroupConvError {
    Status,
    AsOf(ChronoError),
    Currency(String),
    AsOfDateMod,
    ControlTotal { expected: i64, actual: i64 },
    AccountsNum { expected: usize, actual: usize },
    RecordsNum { expected: usize, actual: usize },
}

impl<'a> ast::ParsedGroupHeader<'a> {
    fn convert(&self) -> Result<data::Group, GroupConvError> {
        Ok(data::Group {
            ultimate_receiver: self.ultimate_receiver_ident
                .map(|s| data::Party(s.to_owned())),
            originator: self.originator_ident.map(|s| data::Party(s.to_owned())),
            status: self.status.try_into().or(Err(GroupConvError::Status))?,
            as_of: {
                chrono_date_or_time(&self.as_of_date, self.as_of_time.as_ref())
                    .map_err(GroupConvError::AsOf)?
            },
            currency: self.currency.map_or(Ok(None), |s| {
                s.parse::<penny::Currency>()
                    .map(Some)
                    .map_err(|_| GroupConvError::Currency(s.to_owned()))
            })?,
            as_of_date_mod: self.as_of_date_mod
                .map_or(Ok(None), |m| {
                    m.try_into().or(Err(GroupConvError::AsOfDateMod)).map(Some)
                })?,
            accounts: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum AccountConvError {
    Currency(String),
    AccountInfo(usize, AccountInfoConvError),
    ControlTotal { expected: i64, actual: i64 },
    RecordsNum { expected: usize, actual: usize },
}

impl<'a> ast::ParsedAccountIdent<'a> {
    fn convert(&self) -> Result<(data::Account, i64), AccountConvError> {
        let (infos, control_total) = convert_infos(&self.infos)
            .map_err(|(i, e)| AccountConvError::AccountInfo(i, e))?;
        let account = data::Account {
            customer_account: data::AccountNumber(self.customer_account_num.to_owned()),
            currency: self.currency.map_or(Ok(None), |s| {
                s.parse::<penny::Currency>()
                    .map(Some)
                    .map_err(|_| AccountConvError::Currency(s.to_owned()))
            })?,
            infos: infos,
            transaction_details: Vec::new(),
        };
        Ok((account, control_total))
    }
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
) -> Result<(Vec<data::AccountInfo>, i64), (usize, AccountInfoConvError)> {
    let mut control_total = 0;
    let mut infos = Vec::with_capacity(pinfos.len());
    for (i, pi) in pinfos.iter().enumerate() {
        pi.convert().map_err(|e| (i, e))?.map(|(i, t)| {
            control_total += t;
            infos.push(i);
        });
    }
    Ok((infos, control_total))
}

impl ast::ParsedAccountInfo {
    fn convert(&self) -> Result<Option<(data::AccountInfo, i64)>, AccountInfoConvError> {
        use data::AccountInfo as AI;
        use self::AccountInfoConvError as CE;

        let mut control_total: i64 = 0;
        let info = match (
            self.type_code,
            self.amount,
            self.item_count,
            self.funds_type.as_ref(),
        ) {
            (None, None, None, None) => None,
            (Some(code), amount, item_count, funds) => {
                if let Ok(code) = data::StatusCode::try_from(code) {
                    match (item_count, funds) {
                        (None, None) => {
                            Some(AI::Status {
                                code: code,
                                amount: {
                                    if let Some(a) = amount {
                                        control_total += a;
                                    }
                                    amount
                                },
                            })
                        }
                        (Some(_), _) => return Err(CE::StatusItemCount),
                        (_, Some(_)) => return Err(CE::StatusFunds),
                    }
                } else if let Ok(code) = data::SummaryCode::try_from(code) {
                    Some(AI::Summary {
                        code: code,
                        amount: amount.map_or(Ok(None), |a| if a >= 0 {
                            control_total += a;
                            Ok(Some(a as u64))
                        } else {
                            Err(CE::SummaryNegativeAmount)
                        })?,
                        item_count: item_count,
                        funds: funds
                            .map_or(Ok(None), |f| f.convert().map(Some))
                            .map_err(CE::Funds)?,
                    })
                } else {
                    return Err(CE::InvalidCode);
                }
            }
            _ => return Err(CE::NoCode),
        };
        Ok(info.map(|i| (i, control_total)))
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum FundsTypeConvError {
    ValueDated(ChronoError),
    DistributedAvailDNum(usize),
}

impl ast::ParsedFundsType {
    fn convert(&self) -> Result<data::FundsType, FundsTypeConvError> {
        use ast::ParsedFundsType as PFT;
        use ast::data::FundsType as FT;
        use self::FundsTypeConvError as CE;

        Ok(match *self {
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
                chrono_date_or_time(date, time.as_ref())
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
                        .map(
                            |&ast::ParsedDistributedAvailDistribution { days, amount }| {
                                data::DistributedAvailDistribution { days, amount }
                            },
                        )
                        .collect(),
                )
            }
        })
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub enum TransactionDetailConvError {
    NotTransactionDetailOrAccountTrailer,
    DetailCode(u16),
    Funds(FundsTypeConvError),
}

impl<'a> ast::ParsedTransactionDetail<'a> {
    fn convert(self) -> Result<(data::TransactionDetail, i64), TransactionDetailConvError> {
        let mut control_total: i64 = 0;
        let transaction_detail = data::TransactionDetail {
            code: data::DetailCode::try_from(self.type_code)
                .map_err(TransactionDetailConvError::DetailCode)?,
            amount: {
                if let Some(a) = self.amount {
                    control_total += a;
                }
                self.amount
            },
            funds: self.funds_type
                .as_ref()
                .map_or(Ok(None), |ft| ft.convert().map(Some))
                .map_err(TransactionDetailConvError::Funds)?,
            bank_ref_num: self.bank_ref_num.map(|s| data::ReferenceNum(s.to_owned())),
            customer_ref_num: self.customer_ref_num
                .map(|s| data::ReferenceNum(s.to_owned())),
            text: self.text
                .map(|v| v.into_iter().map(String::from).collect::<Vec<_>>()),
        };
        Ok((transaction_detail, control_total))
    }
}
