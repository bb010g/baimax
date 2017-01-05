use std::marker::PhantomData;
use std::num;
use std::str;
use chrono;
use chrono::{NaiveTime, TimeZone};
use penny;
use regex::Regex;
use void;

use ::data;

macro_rules! raw_parsed {
    () => {};
    ([$(#[$meta:meta])*]; pub struct $raw:ident<$raw_lt:tt> => $parsed:ident {
        $(pub $field:ident: $raw_ty:ty => $parsed_ty:ty,)*
    } $($tail:tt)*) => {
        $(#[$meta])* pub struct $raw<$raw_lt> {
            $(pub $field: $raw_ty,)*
        }
        $(#[$meta])* pub struct $parsed {
            $(pub $field: $parsed_ty,)*
        }
        raw_parsed!($($tail)*);
    };
}
macro_rules! parsed {
    () => {};
    ([$(#[$meta:meta])*]; pub struct $name:ident[$fields:ident]
                                     ($raw:ident<$raw_lt:tt> => $parsed:ident) {
        $(pub $field_ty:ident($field:ident): $raw_ty:ty => $parsed_ty:ty,)*
    } $($tail:tt)*) => {
        #[derive(Debug)]
        pub struct $name<$raw_lt> { phantom: PhantomData<&$raw_lt ()> }
        #[derive(Debug, Copy, Clone)]
        pub enum $fields {
            $($field_ty,)*
        }
        raw_parsed! {
            [#[derive(Debug)] $(#[$meta])*];
            pub struct $raw<$raw_lt> => $parsed {
                $(pub $field: $raw_ty => $parsed_ty,)*
            }
        }
        parsed!($($tail)*);
    };
}

parsed! {
    [];
    pub struct FileHeader[FileHeaderField] (RawFileHeader<'a> => ParsedFileHeader) {
        // 01
        pub SenderIdent(sender_ident): &'a [u8] => String,
        pub ReceiverIdent(receiver_ident): &'a [u8] => String,
        pub CreationDate(creation_date): &'a [u8] => Date,
        pub CreationTime(creation_time): &'a [u8] => Time,
        pub IdentNum(ident_num): &'a [u8] => u32,
        pub PhysicalRecordLen(physical_record_len): Option<&'a [u8]> => Option<u16>,
        pub BlockSize(block_size): Option<&'a [u8]> => Option<u16>,
        pub VersionNumber(version_number): &'a [u8] => (),
    }
    [];
    pub struct GroupHeader[GroupHeaderField] (RawGroupHeader<'a> => ParsedGroupHeader) {
        // 02
        pub UltimateReceiverIdent(ultimate_receiver_ident): Option<&'a [u8]> => Option<String>,
        // Optional because at some banks treat it as such.
        pub OriginatorIdent(originator_ident): Option<&'a [u8]> => Option<String>,
        pub Status(status): &'a [u8] => u8,
        pub AsOfDate(as_of_date): &'a [u8] => Date,
        pub AsOfTime(as_of_time): Option<&'a [u8]> => Option<Time>,
        pub Currency(currency): Option<&'a [u8]> => Option<String>,
        pub AsOfDateMod(as_of_date_mod): Option<&'a [u8]> => Option<u8>,
    }
    [];
    pub struct AccountIdent[AccountIdentField] (RawAccountIdent<'a> => ParsedAccountIdent) {
        // 03
        pub CustomerAccountNum(customer_account_num): &'a [u8] => String,
        pub Currency(currency): Option<&'a [u8]> => Option<String>,
        pub Infos(infos): Vec<RawAccountInfo<'a>> => Vec<ParsedAccountInfo>,
    }
    [];
    pub struct TransactionDetail[TransactionDetailField]
               (RawTransactionDetail<'a> => ParsedTransactionDetail) {
        // 16
        pub TypeCode(type_code): &'a [u8] => u16,
        pub Amount(amount): Option<&'a [u8]> => Option<u64>,
        pub FundsType(funds_type): Option<RawFundsType<'a>> => Option<ParsedFundsType>,
        pub BankRefNum(bank_ref_num): Option<&'a [u8]> => Option<String>,
        pub CustomerRefNum(customer_ref_num): Option<&'a [u8]> => Option<String>,
        pub Text(text): Option<(u8, Vec<&'a [u8]>)> => Option<Vec<String>>,
    }
    [];
    pub struct AccountTrailer[AccountTrailerField] (RawAccountTrailer<'a> => ParsedAccountTrailer) {
        // 49
        pub ControlTotal(control_total): &'a [u8] => i64,
        pub RecordsNum(records_num): &'a [u8] => usize,
    }
    [];
    pub struct GroupTrailer[GroupTrailerField] (RawGroupTrailer<'a> => ParsedGroupTrailer) {
        // 98
        pub ControlTotal(control_total): &'a [u8] => i64,
        pub AccountsNum(accounts_num): &'a [u8] => usize,
        pub RecordsNum(records_num): &'a [u8] => usize,
    }
    [];
    pub struct FileTrailer[FileTrailerField] (RawFileTrailer<'a> => ParsedFileTrailer) {
        // 99
        pub ControlTotal(control_total): &'a [u8] => i64,
        pub GroupsNum(groups_num): &'a [u8] => usize,
        pub RecordsNum(records_num): &'a [u8] => usize,
    }

    [];
    pub struct AccountInfo[AccountInfoField]
               (RawAccountInfo<'a> => ParsedAccountInfo) {
        pub TypeCode(type_code): Option<&'a [u8]> => Option<u16>,
        pub Amount(amount): Option<&'a [u8]> => Option<i64>,
        pub ItemCount(item_count): Option<&'a [u8]> => Option<u32>,
        pub FundsType(funds_type): Option<RawFundsType<'a>> => Option<ParsedFundsType>,
    }
    [];
    pub struct DistributedAvailDistribution[DistributedAvailDistributionField]
               (RawDistributedAvailDistribution<'a> => ParsedDistributedAvailDistribution) {
        pub Days(days): &'a [u8] => u32,
        pub Amount(amount): &'a [u8] => i64,
    }
}

#[derive(Debug)]
pub struct FundsType<'a> {
    phantom: PhantomData<&'a ()>,
}
#[derive(Debug, Copy, Clone)]
pub enum FundsTypeField {
    DistributedAvailSImmediate,
    DistributedAvailSOneDay,
    DistributedAvailSMoreThanOneDay,
    ValueDatedDate,
    ValueDatedTime,
    DistributedAvailDNum,
    DistributedAvailDDists,
}
#[derive(Debug)]
pub enum RawFundsType<'a> {
    Unknown, // Z (default)
    ImmediateAvail, // 0
    OneDayAvail, // 1
    TwoOrMoreDaysAvail, // 2
    DistributedAvailS {
        // S
        // These are optional because the example given treats them as such.
        immediate: Option<&'a [u8]>,
        one_day: Option<&'a [u8]>,
        more_than_one_day: Option<&'a [u8]>,
    },
    ValueDated {
        // V
        date: &'a [u8],
        time: Option<&'a [u8]>,
    },
    DistributedAvailD {
        // D
        num: &'a [u8],
        dists: Vec<RawDistributedAvailDistribution<'a>>,
    },
}
#[derive(Debug)]
pub enum ParsedFundsType {
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
    ValueDated {
        // V
        date: Date,
        time: Option<Time>,
    },
    DistributedAvailD {
        // D
        num: usize,
        dists: Vec<ParsedDistributedAvailDistribution>,
    },
}

#[derive(Debug)]
pub struct Record<'a> {
    phantom: PhantomData<&'a ()>,
}
#[derive(Debug, Copy, Clone)]
pub enum RecordField {
    FileHeader,
    GroupHeader,
    AccountIdent,
    TransactionDetail,
    AccountTrailer,
    GroupTrailer,
    FileTrailer,
}
#[derive(Debug)]
pub enum RawRecord<'a> {
    FileHeader(RawFileHeader<'a>),
    GroupHeader(RawGroupHeader<'a>),
    AccountIdent(RawAccountIdent<'a>),
    TransactionDetail(RawTransactionDetail<'a>),
    AccountTrailer(RawAccountTrailer<'a>),
    GroupTrailer(RawGroupTrailer<'a>),
    FileTrailer(RawFileTrailer<'a>),
}
#[derive(Debug)]
pub enum ParsedRecord {
    FileHeader(ParsedFileHeader),
    GroupHeader(ParsedGroupHeader),
    AccountIdent(ParsedAccountIdent),
    TransactionDetail(ParsedTransactionDetail),
    AccountTrailer(ParsedAccountTrailer),
    GroupTrailer(ParsedGroupTrailer),
    FileTrailer(ParsedFileTrailer),
}

#[derive(Debug)]
pub struct Date {
    pub year: u8,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

pub mod parse;
pub mod convert;
