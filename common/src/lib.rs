pub mod taskmanager;

#[derive(Clone)]
pub struct TicketInfo {
    pub id: String,
    pub name: String,
    pub price: f64,
}
