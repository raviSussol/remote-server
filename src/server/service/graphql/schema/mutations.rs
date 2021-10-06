use async_graphql::Object;

pub struct Mutations;

#[Object]
impl Mutations {
    pub async fn placeholder(&self) -> bool {
        todo!()
    }
}
