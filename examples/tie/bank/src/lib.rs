#[derive(Debug, Clone, PartialEq)]
pub struct Account { pub balance: u64, pub open: bool }

// mitra: implements bank::deposit
pub fn deposit(a: &Account, n: u64) -> Account {
    if a.open { Account { balance: a.balance + n, open: true } } else { a.clone() }
}

impl Account {
    // mitra: implements bank::check
    fn check(&self, n: u64) -> bool { self.open && n <= self.balance }

    /// Take money out.
    // mitra: implements bank::withdraw
    pub fn withdraw(&self, amount: u64) -> (Account, bool) {
        let _ = deposit(self, 0);
        if self.check(amount) { (Account { balance: self.balance - amount, open: true }, true) } else { (self.clone(), false) }
    }
}

#[test]
fn deposit_adds() {
    assert_eq!(deposit(&Account { balance: 0, open: true }, 5), Account { balance: 5, open: true });
}
