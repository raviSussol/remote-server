use async_graphql::{self, dataloader::DataLoader, Context, Enum, ErrorExtensions, Object, Result};
use chrono::{NaiveDate, NaiveDateTime};
use repository::{
    schema::{StocktakeRow, StocktakeStatus},
    unknown_user,
};
use serde::Serialize;

use graphql_core::{
    loader::{InvoiceByIdLoader, StocktakeLineByStocktakeIdLoader, UserAccountLoader},
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};

use super::{InvoiceNode, StocktakeLineConnector, UserNode};

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] // only needed to be comparable in tests
pub enum StocktakeNodeStatus {
    New,
    Finalised,
}

pub struct StocktakeNode {
    pub stocktake: StocktakeRow,
}

#[Object]
impl StocktakeNode {
    pub async fn id(&self) -> &str {
        &self.stocktake.id
    }

    pub async fn store_id(&self) -> &str {
        &self.stocktake.store_id
    }

    /// User that created stocktake, if user is not found in system default unknow user is returned
    pub async fn user(&self, ctx: &Context<'_>) -> Result<UserNode> {
        let loader = ctx.get_loader::<DataLoader<UserAccountLoader>>();

        let user = loader
            .load_one(self.stocktake.user_id.clone())
            .await?
            .unwrap_or(unknown_user());

        Ok(UserNode::from_domain(user))
    }

    pub async fn stocktake_number(&self) -> i64 {
        self.stocktake.stocktake_number
    }

    pub async fn comment(&self) -> &Option<String> {
        &self.stocktake.comment
    }

    pub async fn description(&self) -> &Option<String> {
        &self.stocktake.description
    }

    pub async fn is_locked(&self) -> bool {
        self.stocktake.is_locked
    }

    pub async fn status(&self) -> StocktakeNodeStatus {
        StocktakeNodeStatus::from_domain(&self.stocktake.status)
    }

    pub async fn created_datetime(&self) -> &NaiveDateTime {
        &self.stocktake.created_datetime
    }

    pub async fn stocktake_date(&self) -> &Option<NaiveDate> {
        &self.stocktake.stocktake_date
    }

    pub async fn finalised_datetime(&self) -> &Option<NaiveDateTime> {
        &self.stocktake.finalised_datetime
    }

    pub async fn inventory_adjustment_id(&self) -> &Option<String> {
        &self.stocktake.inventory_adjustment_id
    }

    pub async fn inventory_adjustment(&self, ctx: &Context<'_>) -> Result<Option<InvoiceNode>> {
        if let Some(ref adjustment_id) = self.stocktake.inventory_adjustment_id {
            let loader = ctx.get_loader::<DataLoader<InvoiceByIdLoader>>();
            let invoice = loader.load_one(adjustment_id.clone()).await?.ok_or(
                StandardGraphqlError::InternalError(format!(
                    "Cannot find inventory adjustment {}",
                    adjustment_id
                ))
                .extend(),
            )?;
            Ok(Some(InvoiceNode { invoice }))
        } else {
            Ok(None)
        }
    }

    pub async fn lines(&self, ctx: &Context<'_>) -> Result<StocktakeLineConnector> {
        let loader = ctx.get_loader::<DataLoader<StocktakeLineByStocktakeIdLoader>>();

        let lines_option = loader.load_one(self.stocktake.id.clone()).await?;

        let result = match lines_option {
            None => StocktakeLineConnector::empty(),
            Some(lines) => StocktakeLineConnector::from_domain_vec(lines),
        };

        Ok(result)
    }
}

impl StocktakeNodeStatus {
    pub fn to_domain(self) -> StocktakeStatus {
        match self {
            StocktakeNodeStatus::New => StocktakeStatus::New,
            StocktakeNodeStatus::Finalised => StocktakeStatus::Finalised,
        }
    }

    pub fn from_domain(status: &StocktakeStatus) -> StocktakeNodeStatus {
        match status {
            StocktakeStatus::New => StocktakeNodeStatus::New,
            StocktakeStatus::Finalised => StocktakeNodeStatus::Finalised,
        }
    }
}

#[cfg(test)]
mod test {
    use async_graphql::{EmptyMutation, Object};

    use graphql_core::{assert_graphql_query, test_helpers::setup_graphl_test};
    use repository::{
        mock::{mock_user_account_a, MockDataInserts},
        schema::StocktakeRow,
        unknown_user,
    };
    use serde_json::json;
    use util::inline_init;

    use crate::types::StocktakeNode;

    #[actix_rt::test]
    async fn graphql_stocktake_user_loader() {
        #[derive(Clone)]
        struct TestQuery;

        let (_, _, _, settings) = setup_graphl_test(
            TestQuery,
            EmptyMutation,
            "graphql_stocktake_user_loader",
            MockDataInserts::none().user_accounts(),
        )
        .await;

        #[Object]
        impl TestQuery {
            pub async fn test_query_user_exists(&self) -> StocktakeNode {
                StocktakeNode {
                    stocktake: inline_init(|r: &mut StocktakeRow| {
                        r.user_id = mock_user_account_a().id;
                    }),
                }
            }
            pub async fn test_query_user_does_not_exist(&self) -> StocktakeNode {
                StocktakeNode {
                    stocktake: inline_init(|r: &mut StocktakeRow| {
                        r.user_id = "does not exist".to_string()
                    }),
                }
            }
        }

        let expected = json!({
            "testQueryUserExists": {
                "user": {
                    "userId": mock_user_account_a().id
                }
            },
            "testQueryUserDoesNotExist": {
                "user": {
                    "userId": unknown_user().id
                }
            },
        }
        );

        let query = r#"
        query {
            testQueryUserExists {
                ...user
            }
            testQueryUserDoesNotExist {
                ...user
            }         
        }
        fragment user on StocktakeNode {
            user {
                userId
            }
        }
        "#;

        assert_graphql_query!(&settings, &query, &None, expected, None);
    }
}
