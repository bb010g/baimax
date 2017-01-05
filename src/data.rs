use chrono::{Date, DateTime, TimeZone};
use penny::Currency;
pub use type_codes::*;

#[derive(Debug)]
pub enum DateOrTime<Tz: TimeZone> {
    Date(Date<Tz>),
    DateTime(DateTime<Tz>),
}

#[derive(Debug)]
pub struct File<'cur, Tz: TimeZone> {
    pub sender: Party,
    pub receiver: Party,
    pub creation: DateTime<Tz>,
    pub ident: FileIdent,
    pub groups: Vec<Group<'cur, Tz>>,
}

#[derive(Debug)]
pub struct Party(pub String);
#[derive(Debug)]
pub struct FileIdent(pub u32);

#[derive(Debug)]
pub struct Group<'cur, Tz: TimeZone> {
    pub ultimate_receiver: Option<Party>,
    // Optional because at least HomeStreet treats it as such.
    pub originator: Option<Party>,
    pub status: GroupStatus,
    pub as_of: DateOrTime<Tz>,
    pub currency: Option<&'cur Currency<'cur>>,
    pub as_of_date_mod: Option<AsOfDateModifier>,
    pub accounts: Vec<Account<'cur, Tz>>,
}


enum_mapping! {
    #[derive(Debug)]
    pub GroupStatus(u8) {
        Update(1),
        Deletion(2),
        Correction(3),
        TestOnly(4),
    }
}

enum_mapping! {
    #[derive(Debug)]
    pub AsOfDateModifier(u8) {
        InterimPrevious(1),
        FinalPrevious(2),
        InterimSame(3),
        FinalSame(4),
    }
}

#[derive(Debug)]
pub struct Account<'cur, Tz: TimeZone> {
    pub customer_account: AccountNumber,
    pub currency: Option<&'cur Currency<'cur>>,
    pub infos: Vec<AccountInfo<Tz>>,
    pub transaction_details: Vec<TransactionDetail<Tz>>,
}

#[derive(Debug)]
pub enum AccountInfo<Tz: TimeZone> {
    Summary {
        code: SummaryCode,
        amount: Option<u64>,
        item_count: Option<u32>,
        funds: Option<FundsType<Tz>>,
    },
    Status {
        code: StatusCode,
        amount: Option<i64>,
    },
}

#[derive(Debug)]
pub struct AccountNumber(pub String);

#[derive(Debug)]
pub enum FundsType<Tz: TimeZone> {
    Unknown, // Z (default)
    ImmediateAvail, // 0
    OneDayAvail, // 1
    TwoOrMoreDaysAvail, // 2
    DistributedAvailS {
        // S
        // These are optional because the example given treats them as such.
        immediate: Option<i64>,
        one_day: Option<i64>,
        more_than_one_day: Option<i64>,
    },
    ValueDated(DateOrTime<Tz>), // V
    DistributedAvailD(Vec<DistributedAvailDistribution>), // D
}

#[derive(Debug)]
pub struct DistributedAvailDistribution {
    pub days: u32,
    pub amount: i64,
}

#[derive(Debug)]
pub struct TransactionDetail<Tz: TimeZone> {
    pub code: DetailCode,
    pub amount: Option<u64>,
    pub funds: Option<FundsType<Tz>>,
    pub bank_ref_num: Option<ReferenceNum>,
    pub customer_ref_num: Option<ReferenceNum>,
    pub text: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ReferenceNum(pub String);
