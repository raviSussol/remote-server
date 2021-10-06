use diesel::{
    dsl::{InnerJoin, IntoBoxed, LeftJoin},
    prelude::*,
    query_builder::{AsQuery, BoxedSelectStatement},
    query_source::joins::{Inner, Join, JoinOn, LeftOuter},
};

use super::DBType;

// BOXED QUERY

// Boxed Query - Left Join For Two Tables

type LeftJoinSelect<FromTable, ToTable> = <LeftJoin<FromTable, ToTable> as AsQuery>::SqlType;

type LeftJoinOn<FromTable, ToTable> = <FromTable as JoinTo<ToTable>>::OnClause;

type LeftJoinQuery<FromTable, ToTable> =
    JoinOn<Join<FromTable, ToTable, LeftOuter>, LeftJoinOn<FromTable, ToTable>>;

pub type LeftJoinBoxedStatement<'a, FromTable, ToTable> = BoxedSelectStatement<
    'a,
    LeftJoinSelect<FromTable, ToTable>,
    LeftJoinQuery<FromTable, ToTable>,
    DBType,
>;

// Boxed Query - Left Join For Three Tables

type LeftJoinSelectThreeWay<FromTable, FirstToTable, SecondToTable> =
    <LeftJoin<FromTable, LeftJoin<FirstToTable, SecondToTable>> as AsQuery>::SqlType;

type LeftJoinQueryThreeWay<FromTable, FirstToTable, SecondToTable> = JoinOn<
    Join<FromTable, LeftJoin<FirstToTable, SecondToTable>, LeftOuter>,
    LeftJoinOn<FromTable, FirstToTable>,
>;

pub type LeftJoinBoxedStatementThreeWay<'a, FromTable, FirstToTable, SecondToTable> =
    BoxedSelectStatement<
        'a,
        LeftJoinSelectThreeWay<FromTable, FirstToTable, SecondToTable>,
        LeftJoinQueryThreeWay<FromTable, FirstToTable, SecondToTable>,
        DBType,
    >;

// Boxed Query - Left Join For Four Tables

type LeftJoinSelectFourWay<FromTable, FirstToTable, SecondToTable, ThirdToTable> = <LeftJoin<
    FromTable,
    LeftJoin<FirstToTable, LeftJoin<SecondToTable, ThirdToTable>>,
> as AsQuery>::SqlType;

type LeftJoinQueryFourWay<FromTable, FirstToTable, SecondToTable, ThirdToTable> = JoinOn<
    Join<FromTable, LeftJoin<FirstToTable, LeftJoin<SecondToTable, ThirdToTable>>, LeftOuter>,
    LeftJoinOn<FromTable, FirstToTable>,
>;

pub type LeftJoinBoxedStatementFourWay<'a, FromTable, FirstToTable, SecondToTable, ThirdToTable> =
    BoxedSelectStatement<
        'a,
        LeftJoinSelectFourWay<FromTable, FirstToTable, SecondToTable, ThirdToTable>,
        LeftJoinQueryFourWay<FromTable, FirstToTable, SecondToTable, ThirdToTable>,
        DBType,
    >;

// Boxed Query - Inner Join Three Tables

type InnerJoinSelectThreeWay<FromTable, FirstToTable, SecondToTable> =
    <InnerJoin<InnerJoin<FromTable, FirstToTable>, SecondToTable> as AsQuery>::SqlType;

type InnerJoinQueryThreeWay<FromTable, FirstToTable, SecondToTable> =
    InnerJoin<InnerJoin<FromTable, FirstToTable>, SecondToTable>;

// pub type InnerJoinBoxedStatementThreeWay<'a, FromTable, FirstToTable, SecondToTable> =
//     BoxedSelectStatement<
//         'a,
//         InnerJoinSelectThreeWay<FromTable, FirstToTable, SecondToTable>,
//         InnerJoinQueryThreeWay<FromTable, FirstToTable, SecondToTable>,
//         DBType,
//     >;

pub type InnerJoinBoxedStatementThreeWay<'a, FromTable, FirstToTable, SecondToTable> =
    IntoBoxed<'a, InnerJoin<InnerJoin<FromTable, FirstToTable>, SecondToTable>, DBType>;
