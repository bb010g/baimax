#![cfg_attr(feature="lint", allow(enum_variant_names))]

use std::convert::{TryFrom, TryInto};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
pub enum TypeCode {
    Status(StatusCode),
    Summary(SummaryCode),
    Detail(DetailCode),
}
impl fmt::Display for TypeCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Type(")?;
        match *self {
            TypeCode::Status(ref code) => write!(f, "Status, {})", code),
            TypeCode::Summary(ref code) => write!(f, "Summary, {})", code),
            TypeCode::Detail(ref code) => write!(f, "Detail, {})", code),
        }
    }
}

impl From<TypeCode> for u16 {
    fn from(code: TypeCode) -> u16 {
        match code {
            TypeCode::Status(c) => c.into(),
            TypeCode::Summary(c) => c.into(),
            TypeCode::Detail(c) => c.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
pub enum StatusCode {
    // 001-099, 900-919
    Account(AccountStatus),
    // 700-719
    Loan(LoanStatus),
}

impl TryFrom<u16> for StatusCode {
    type Error = u16;
    fn try_from(code: u16) -> Result<StatusCode, u16> {
        match code {
            c @ 1...99 | c @ 900...919 => c.try_into().map(StatusCode::Account),
            c @ 700...719 => c.try_into().map(StatusCode::Loan),
            _ => Err(code),
        }
    }
}
impl From<StatusCode> for u16 {
    fn from(code: StatusCode) -> u16 {
        match code {
            StatusCode::Account(c) => c.into(),
            StatusCode::Loan(c) => c.into(),
        }
    }
}
impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Status(")?;
        match *self {
            StatusCode::Account(ref code) => write!(f, "Account, {:?})", code),
            StatusCode::Loan(ref code) => write!(f, "Loan, {:?})", code),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
pub enum SummaryCode {
    // 100-399, 920-959
    Credit(CreditSummary),
    // 400-699, 960-999
    Debit(DebitSummary),
    // 700-799
    Loan(LoanSummary),
}

impl TryFrom<u16> for SummaryCode {
    type Error = u16;
    fn try_from(code: u16) -> Result<SummaryCode, u16> {
        match code {
            c @ 100...399 | c @ 920...959 => c.try_into().map(SummaryCode::Credit),
            c @ 400...469 | c @ 960...999 => c.try_into().map(SummaryCode::Debit),
            c @ 700...799 => c.try_into().map(SummaryCode::Loan),
            _ => Err(code),
        }
    }
}
impl From<SummaryCode> for u16 {
    fn from(code: SummaryCode) -> u16 {
        match code {
            SummaryCode::Credit(c) => c.into(),
            SummaryCode::Debit(c) => c.into(),
            SummaryCode::Loan(c) => c.into(),
        }
    }
}
impl fmt::Display for SummaryCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Summary(")?;
        match *self {
            SummaryCode::Credit(ref code) => write!(f, "Credit, {:?})", code),
            SummaryCode::Debit(ref code) => write!(f, "Debit, {:?})", code),
            SummaryCode::Loan(ref code) => write!(f, "Loan, {:?})", code),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
pub enum DetailCode {
    // 100-399, 920-959
    Credit(CreditDetail),
    // 400-699, 960-999
    Debit(DebitDetail),
    // 700-799
    Loan(LoanDetail),
    // 890
    NonMonetary,
}

impl TryFrom<u16> for DetailCode {
    type Error = u16;
    fn try_from(code: u16) -> Result<DetailCode, u16> {
        match code {
            c @ 100...399 | c @ 920...959 => c.try_into().map(DetailCode::Credit),
            c @ 400...699 | c @ 960...999 => c.try_into().map(DetailCode::Debit),
            c @ 700...799 => c.try_into().map(DetailCode::Loan),
            890 => Ok(DetailCode::NonMonetary),
            _ => Err(code),
        }
    }
}
impl From<DetailCode> for u16 {
    fn from(code: DetailCode) -> u16 {
        match code {
            DetailCode::Credit(c) => c.into(),
            DetailCode::Debit(c) => c.into(),
            DetailCode::Loan(c) => c.into(),
            DetailCode::NonMonetary => 890,
        }
    }
}
impl fmt::Display for DetailCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Summary(")?;
        match *self {
            DetailCode::Credit(ref code) => write!(f, "Credit, {:?})", code),
            DetailCode::Debit(ref code) => write!(f, "Debit, {:?})", code),
            DetailCode::Loan(ref code) => write!(f, "Loan, {:?})", code),
            DetailCode::NonMonetary => write!(f, "NonMonetary)"),
        }
    }
}

enum_mapping! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
    pub AccountStatus(u16) {
        OpeningLedger(10),
        AvgOpeningLedgerMtd(11),
        AvgOpeningLedgerYtd(12),
        ClosingLedger(15),
        AvgClosingLedgerMtd(20),
        AvgClosingLedgerPrevMonth(21),
        AggregateBalanceAdjustments(22),
        AvgClosingLedgerYtdPrevMonth(24),
        AvgClosingLedgerYtd(25),
        CurrentLedger(30),
        AchNetPosition(37),
        OpeningAvailPlusTotalSameDayAchDtcDeposit(39),
        OpeningAvail(40),
        AvgOpeningAvailMtd(41),
        AvgOpeningAvailYtd(42),
        AvgAvailPrevMonth(43),
        DisbursingOpeningAvailBalance(44),
        ClosingAvail(45),
        AvgClosingAvailMtd(50),
        AvgClosingAvailLastMonth(51),
        AvgClosingAvailYtdLastMonth(54),
        AvgClosingAvailYtd(55),
        LoanBalance(56),
        TotalInvestmentPosition(57),
        CurrentAvailCrsSupressed(59),
        CurrentAvail(60),
        AvgCurrentAvailMtd(61),
        AvgCurrentAvailYtd(62),
        TotalFloat(63),
        TargetBalance(65),
        AdjustedBalance(66),
        AdjustedBalanceMtd(67),
        AdjustedBalanceYtd(68),
        ZeroDayFloat(70),
        OneDayFloat(72),
        FloatAdjustment(73),
        TwoOrMoreDaysFloat(74),
        ThreeOrMoreDaysFloat(75),
        AdjustmentToBalances(76),
        AvgAdjustmentToBalancesMtd(77),
        AvgAdjustmentToBalancesYtd(78),
        FourDayFloat(79),
        FiveDayFloat(80),
        SixDayFloat(81),
        AvgOneDayFloatMtd(82),
        AvgOneDayFloatYtd(83),
        AvgTwoDayFloatMtd(84),
        AvgTwoDayFloatYtd(85),
        TransferCalculation(86);

        Custom {
            from: c @ 900...919 => Ok(AccountStatus::Custom(c));
            to: AccountStatus::Custom(c) => c;
        }
    }
}

enum_mapping! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
    pub LoanStatus(u16) {
        PrincipalLoanBalance(701),
        AvailableCommitmentAmount(703),
        PaymentAmountDue(705),
        PrincipalAmountPastDue(707),
        InterestAmountPastDue(709),
    }
}

enum_mapping! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
    pub CreditSummary(u16) {
        TotalCredits(100),
        TotalCreditAmountMtd(101),
        CreditsNotDetailed(105),
        DespositsSubjectToFloat(106),
        TotalAdjustmentCreditsYtd(107),
        CurrentDayTotalLockboxDeposits(109),
        // Lockbox
        TotalLockboxDeposits(110),
        EdiTransactionCredit(120),
        // Concentration
        TotalConcentrationCredits(130),
        TotalDtcCredits(131),
        TotalAchCredits(140),
        TotalBankCardDeposits(146),
        // Preauthorized and ACH
        TotalPreauthorizedPaymentCredits(150),
        TotalAchDisbursingFundingCredits(160),
        CorporateTradePaymentSettlement(162),
        CorporateTradePaymentCredits(163),
        AchSettlementCredits(167),
        // Other Deposits
        TotalOtherCheckDeposits(170),
        ListPostCredits(178),
        TotalLoanProceeds(180),
        TotalBankPreparedDeposits(182),
        TotalMiscDeposits(185),
        TotalCashLetterCredits(186),
        TotalCashLetterAdjustments(188),
        // Money Transfer
        TotalIncomingMoneyTransfers(190),
        TotalAutomaticTransferCredits(200),
        TotalBookTransferCredits(205),
        TotalInternationalMoneyTransferCredits(207),
        TotalInternationalCredits(210),
        TotalLettersOfCredit(215),
        // Security
        TotalSecurityCredits(230),
        TotalCollectionCredits(231),
        TotalBankersAcceptanceCredits(239),
        MonthlyDividends(245),
        TotalChecksPostedAndReturned(250),
        TotalDebitReversals(251),
        TotalAchReturnItems(256),
        TotalRejectedCredits(260),
        // ZBA and Disbursing
        TotalZbaCredits(270),
        NetZeroBalanceAmount(271),
        TotalControlledDisbursingCredits(280),
        TotalDtcDisbursingCredits(285),
        // Other (Expansion)
        TotalAtmCredits(294),
        CorrespondentBankDeposit(302),
        TotalWireTransfersInFf(303),
        TotalWireTransfersInChf(304),
        TotalFedFundsSold(305),
        TotalTrustCredits(307),
        TotalValueDatedFunds(309),
        TotalCommercialDeposits(310),
        TotalInternationalCreditsFf(315),
        TotalInternationalCreditsChf(316),
        TotalForeignCheckPurchased(318),
        LateDeposit(319),
        TotalSecuritiesSoldFf(320),
        TotalSecuritiesSoldChf(321),
        TotalSecuritiesMaturedFf(324),
        TotalSecuritiesMaturedChf(325),
        TotalSecuritiesInterest(326),
        TotalSecuritiesMatured(327),
        TotalSecuritiesInterestFf(328),
        TotalSecuritiesInterestChf(329),
        TotalEscrowCredits(330),
        TotalMiscSecuritiesCreditsFf(332),
        TotalMiscSecuritiesCreditsChf(336),
        TotalSecuritiesSold(338),
        TotalBrokerDeposits(340),
        TotalBrokerDepositsFf(341),
        TotalBrokerDepositsChf(343),
        InvestmentSold(350),
        TotalCashCenterCredits(352),
        InvestmentInterest(355),
        TotalCreditAdjustment(356),
        TotalCreditsLessWireTransferAndReturnedChecks(360),
        GrandTotalCreditsLessGrandTotalDebits(361),
        // Correspondent Bank and Federal Reserve
        TotalBackValueCredits(370),
        TotalUniversalCredits(385),
        TotalFreightPaymentCredits(389),
        // Miscellaneous
        TotalMiscCredits(390);

        Custom {
            from: c @ 920...959 => Ok(CreditSummary::Custom(c));
            to: CreditSummary::Custom(c) => c;
        }
    }
}

enum_mapping! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
    pub DebitSummary(u16) {
        TotalDebits(400),
        TotalDebitAmountMtd(401),
        TodaysTotalDebits(403),
        TotalDebitLessWireTransfersAndChargeBacks(405),
        DebitsNotDetailed(406),
        TotalYtdAdjustment(410),
        TotalDebitsExcludingReturnedItems(412),
        // Lockbox
        TotalLockboxDebits(416),
        EdiTransactionDebits(420),
        // Payable-Through Draft
        TotalPayableThroughDrafts(430),
        // ACH
        TotalAchDisbursementFundingDebits(446),
        TotalAchDebits(450),
        CorporateTradePaymentDebits(463),
        CorporateTradePaymentSettlement(465),
        AchSettlementDebits(467),
        // Checks Paid
        TotalCheckPaid(470),
        TotalCheckPaidCumulativeMtd(471),
        ListPostDebits(478),
        TotalLoanPayments(480),
        TotalBankOriginatedDebits(482),
        TotalCashLetterDebits(486),
        // Money Transfer
        TotalOutgoingMoneyTransfers(490),
        TotalAutomaticTransferDebits(500),
        TotalBookTransferDebits(505),
        TotalInternationalMoneyTransferDebits(507),
        TotalInternationalDebits(510),
        TotalLettersOfCredit(515),
        // Security
        TotalSecurityDebits(530),
        TotalAmountOfSecuritiesPurchased(532),
        TotalMiscSecuritiesDbFf(534),
        TotalMiscSecuritiesDebitChf(536),
        TotalCollectionDebit(537),
        TotalBankersAcceptancesDebit(539),
        // Deposited Items Returned
        TotalDepositedItemsReturned(550),
        TotalCreditReversals(551),
        TotalAchReturnItems(556),
        TotalRejectedDebits(560),
        // ZBA and Disbursing
        TotalZbaDebits(570),
        TotalControlledDisbursingDebits(580),
        TotalDisbursingChecksPaidEarlyAmount(583),
        TotalDisbursingChecksPaidLaterAmount(584),
        DisbursingFundingRequirement(585),
        FrbPresentmentEstimateFedEstimate(586),
        LateDebitsAfterNotification(587),
        TotalDisbursingChecksPaidLastAmount(588),
        // Other (Expansion)
        TotalDtcDebits(590),
        TotalAtmDebits(594),
        TotalAprDebits(596),
        EstimatedTotalDisbursement(601),
        AdjustedTotalDisbursement(602),
        TotalFundsRequired(610),
        TotalWireTransfersOutChf(611),
        TotalWireTransfersOutFf(612),
        TotalInternationalDebitChf(613),
        TotalInternationalDebitFf(614),
        TotalFederalReserveBankCommercialBankDebit(615),
        TotalSecuritiesPurchasedChf(617),
        TotalSecuritiesPurchasedFf(618),
        TotalBrokerDebitsChf(621),
        TotalBrokerDebitsFf(623),
        TotalBrokerDebits(625),
        TotalFedFundsPurchased(626),
        TotalCashCenterDebits(628),
        TotalDebitAdjustments(630),
        TotalTrustDebits(632),
        TotalEscrowDebits(640),
        TransferCalculationDebit(646),
        InvestmentsPurchased(650),
        TotalInvestmentInterestDebits(655),
        // Correspondent Bank and Federal Reserve
        InterceptDebits(665),
        TotalBackValueDebits(670),
        TotalUniversalDebits(685),
        FrbFreightPaymentDebits(689);

        Custom {
            from: c @ 960...999 => Ok(DebitSummary::Custom(c));
            to: DebitSummary::Custom(c) => c;
        }
    }
}

enum_mapping! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
    pub LoanSummary(u16) {
        TotalLoanPayment(720),
        LoanDisbursement(760),
    }
}

enum_mapping! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
    pub CreditDetail(u16) {
        CreditAnyType(108),
        // Lockbox
        LockboxDeposit(115),
        ItemInLockboxDeposit(116),
        LockboxAdjustmentCredit(118),
        EdiTransactionCredit(121),
        EdibanxCreditReceived(122),
        EdibanxCreditReturn(123),
        // Concentration
        DtcConcentrationCredit(135),
        ItemInDtcDeposit(136),
        AchCreditReceived(142),
        ItemInAchDeposit(143),
        AchConcentrationCredit(145),
        IndividualBankCardDeposit(147),
        // Preauthorized and ACH
        PreauthorizedDraftCredit(155),
        ItemInPacDeposit(156),
        CorporateTradePaymentCredit(164),
        PreauthorizedAchCredit(165),
        AchSettlement(166),
        AchReturnItemOrAdjustmentSettlement(168),
        MiscAchCredit(169),
        // Other Deposits
        IndividualLoanDeposit(171),
        DepositCorrection(172),
        BankPreparedDeposit(173),
        OtherDeposit(174),
        CheckDepositPackage(175),
        RePresentedCheckDeposit(176),
        DraftDeposit(184),
        CashLetterCredit(187),
        CashLetterAdjustment(189),
        // Money Transfer
        IndividualIncomingInternalMoneyTransfer(191),
        IncomingMoneyTransfer(195),
        MoneyTransferAdjustment(196),
        Compensation(198),
        IndividualAutomaticTransferCredit(201),
        BondOperationsCredit(202),
        BookTransferCredit(206),
        IndividualInternationalMoneyTransferCredit(208),
        ForeignLetterOfCredit(212),
        LetterOfCredit(213),
        ForeignExchangeOfCredit(214),
        ForeignRemittanceCredit(216),
        ForeignCollectionCredit(218),
        ForeignCheckPurchase(221),
        ForeignChecksDeposited(222),
        Commission(224),
        InternationalMoneyMarketTrading(226),
        StandingOrder(227),
        MiscInternationalCredit(229),
        // Security
        SaleOfDebtSecurity(232),
        SecuritiesSold(233),
        SaleOfEquitySecurity(234),
        MaturedReverseRepurchaseOrder(235),
        MaturityOfDebtSecurity(236),
        IndividualCollectionCredit(237),
        CollectionOfDividends(238),
        CouponCollectionsBanks(240),
        BankersAcceptances(241),
        CollectionOfInterestIncome(242),
        MaturedFedFundsPurchased(243),
        InterestOrMaturedPrincipalPayment(244),
        CommercialPaper(246),
        CapitalChange(247),
        SavingsBondsSalesAdjustment(248),
        MiscSecurityCredit(249),
        DebitReversal(252),
        PostingErrorCorrectionCredit(254),
        CheckPostedAndReturned(255),
        IndividualAchReturnItem(257),
        AchReversalCredit(258),
        IndividualRejectedCredit(261),
        Overdraft(263),
        ReturnItem(266),
        ReturnItemAdjustment(268),
        // ZBA and Disbursing
        CumulativeZbaOrDisbursementCredits(274),
        ZbaCredit(275),
        ZbaFloatAdjustment(276),
        ZbaCreditTransfer(277),
        ZbaCreditAdjustment(278),
        IndividualControlledDisbursingCredit(281),
        IndividualDtcDisbursingCredit(286),
        // Other (Expansion)
        AtmCredit(295),
        CommercialDeposit(301),
        FedFundsSold(306),
        TrustCredit(308),
        IndividualEscrowCredit(331),
        BrokerDeposit(342),
        IndividualBackValueCredit(344),
        ItemInBrokersDeposit(345),
        SweepInterestIncome(346),
        SweepPrincipalSell(347),
        FuturesCredit(348),
        PrincipalPaymentsCredit(349),
        IndividualInvestmentSold(351),
        CashCenterCredit(353),
        InterestCredit(354),
        CreditAdjustment(357),
        YtdAdjustmentCredit(358),
        InterestAdjustmentCredit(359),
        // Correspondent Bank and Federal Reserve
        CorrespondentCollection(362),
        CorrespondentCollectionAdjustment(363),
        LoanParticipation(364),
        CurrencyAndCoinDeposited(366),
        FoodStampLetter(367),
        FoodStampAdjustment(368),
        ClearingSettlementCredit(369),
        BackValueAdjustment(372),
        CustomerPayroll(373),
        FrbStatementRecap(374),
        SavingsBondLetterOrAdjustment(376),
        TreasuryTaxAndLoanCredit(377),
        TransferOfTreasuryCredit(378),
        FrbGovernmentChecksCashLetterCredit(379),
        FrbGovernmentCheckAdjustment(381),
        FrbPostalMoneyOrderCredit(382),
        FrbPostalMoneyOrderAdjustment(383),
        FrbCashLetterAutoChargeCredit(384),
        FrbCashLetterAutoChargeAdjustment(386),
        FrbFineSortCashLetterCredit(387),
        FrbFineSortAdjustment(388),
        // Miscellaneous
        UniversalCredit(391),
        FreightPaymentCredit(392),
        ItemizedCreditOverTenThousandDollars(393),
        CumulativeCredits(394),
        CheckReversal(395),
        FloatAdjustment(397),
        MiscFeeRefund(398),
        MiscCredit(399);

        // 920-959
        Custom {
            from: c @ 920...959 => Ok(CreditDetail::Custom(c));
            to: CreditDetail::Custom(c) => c;
        }
    }
}

enum_mapping! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
    pub DebitDetail(u16) {
        FloatAdjustment(408),
        DebitAnyType(409),
        // Lockbox
        LockboxDebit(415),
        EdiTransactionDebit(421),
        EdibanxSettlementDebit(422),
        EdibanxReturnItemDebit(423),
        // Payable-Through Draft
        PayableThroughDraft(435),
        // ACH
        AchConcentrationDebit(445),
        AchDisbursementFundingDebit(447),
        AchDebitRecieved(451),
        ItemInAchDisbursementOrDebit(452),
        PreauthorizedAchDebit(455),
        AccountHolderInitiatedAchDebit(462),
        CorporateTradePaymentDebit(464),
        AchSettlement(466),
        AchReturnItemOrAdjustmentSettlement(468),
        MiscAchDebit(469),
        // Checks Paid
        CumulativeChecksPaid(472),
        CertifiedCheckDebit(474),
        CheckPaid(475),
        FederalReserveBankLetterDebit(476),
        BankOriginatedDebit(477),
        ListPostDebit(479),
        IndividualLoanPayment(481),
        Draft(484),
        DtcDebit(485),
        CashLetterDebit(487),
        CashLetterAdjustment(489),
        // Money Transfer
        IndividualOutgoingInternalMoneyTransfer(491),
        CustomerTerminalInitiatedMoneyTransfer(493),
        OutgoingMoneyTransfer(495),
        MoneyTransferAdjustment(496),
        Compensation(498),
        IndividualAutomaticTransferDebit(501),
        BondOperationsDebit(502),
        BookTransferDebit(506),
        IndividualInternationalMoneyTransferDebits(507),
        LetterOfCreditDebit(512),
        LetterOfCredit(513),
        ForeignExchangeDebit(514),
        ForeignRemittanceDebit(516),
        ForeignCollectionDebit(518),
        ForeignChecksPaid(522),
        Commision(524),
        InternationalMoneyMarketTrading(526),
        StandingOrder(527),
        MiscInternationalDebit(529),
        // Security
        SecuritiesPurchased(531),
        SecurityCollectionDebit(533),
        PurchaseOfEquitySecurities(535),
        MaturedRepurchaseOrder(538),
        CouponCollectionDebit(540),
        BankersAcceptances(541),
        PurchaseOfDebtSecurities(542),
        DomesticCollection(543),
        InterestOrMaturedPrincipalPayment(544),
        CommercialPaper(546),
        CapitalChange(547),
        SavingsBondsSalesAdjustment(548),
        MiscSecurityDebit(549),
        // Deposited Items Returned
        CreditReversal(552),
        PostingErrorCorrectionDebit(554),
        DepositedItemReturned(555),
        IndividualAchReturnItem(557),
        AchReversalDebit(558),
        IndividualRejectedDebit(561),
        Overdraft(563),
        OverdraftFee(564),
        ReturnItem(566),
        ReturnItemFee(567),
        ReturnItemAdjustment(568),
        // ZBA and Disbursing
        CumulativeZbaDebits(574),
        ZbaDebit(575),
        ZbaDebitTransfer(577),
        ZbaDebitAdjustment(578),
        IndividualControlledDisbursingDebit(581),
        // Other (Expansion)
        AtmDebit(595),
        ArpDebit(597),
        FederalReserveBankCommercialBankDebit(616),
        BrokerDebit(622),
        FedFundsPurchased(627),
        CashCenterDebit(629),
        DebitAdjustment(631),
        TrustDebit(633),
        YtdAdjustmentDebit(634),
        IndividualEscrowDebit(641),
        IndividualBackValueDebit(644),
        IndividualInvestmentPurchased(651),
        InterestDebit(654),
        SweepPrincipalBuy(656),
        FuturesDebit(657),
        PrincipalPaymentsDebit(658),
        InterestAdjustmentDebit(659),
        // Correspondent Bank and Federal Reserve
        AccountAnalysisFee(661),
        CorrespondentCollectionDebit(662),
        CorrespondentCollectionAdjustment(663),
        LoanParticipation(664),
        CurrencyAndCoinShipped(666),
        FoodStampLetter(667),
        FoodStampAdjustment(668),
        ClearingSettlementDebit(669),
        BackValueAdjustment(672),
        CustomerPayroll(673),
        FrbStatementRecap(674),
        SavingsBondLetterOrAdjustment(676),
        TreasuryTaxAndLoanDebit(677),
        TransferOfTreasuryDebit(678),
        FrbGovernmentChecksCashLetterDebit(679),
        FrbGovernmentCheckAdjustment(681),
        FrbPostalMoneyOrderDebit(682),
        FrbPostalMoneyOrderAdjustment(683),
        FrbCashLetterAutoChargeDebit(684),
        FrbCashLetterAutoChargeAdjustment(686),
        FrbFineSortCashLetterDebit(687),
        FrbFineSortAdjustment(688),
        UniversalDebit(691),
        FreightPaymentDebit(692),
        ItemizedDebitOverTenThousandDollars(693),
        DepositReversal(694),
        DepositCorrectionDebit(695),
        RegularCollectionDebit(696),
        CumulativeDebits(697),
        MiscFees(698),
        MiscDebit(699);

        // 960-999
        Custom {
            from: c @ 960...999 => Ok(DebitDetail::Custom(c));
            to: DebitDetail::Custom(c) => c;
        }
    }
}

enum_mapping! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature="serde-serialize", derive(Serialize, Deserialize))]
    pub LoanDetail(u16) {
        AmountAppliedToInterest(721),
        AmountAppliedToPrincipal(722),
        AmountAppliedToEscrow(723),
        AmountAppliedToLateCharges(724),
        AmountAppliedToBuydown(725),
        AmountAppliedToMiscFees(726),
        AmountAppliedToDeferredInterestDetail(727),
        AmountAppliedToServiceCharge(728),
    }
}
