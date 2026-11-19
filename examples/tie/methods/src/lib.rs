// mitra: implements bank::Account
pub struct Account {
    pub balance: i64,
    pub owner: String,
}

impl Account {
    // mitra: implements bank::Account::open
    pub fn open() -> Self {
        Account { balance: 0, owner: String::new() }
    }

    // mitra: implements bank::Account::deposit
    pub fn deposit(mut self, amount: i64) -> Self {
        self.balance += amount;
        self
    }

    // mitra: implements bank::Account::can_pay
    pub fn can_pay(&self, amount: i64) -> bool {
        self.balance >= amount
    }

    // The model says it gives an `Int`.
    // mitra: implements bank::Account::close
    pub fn close(self) -> Self {
        self
    }
}
