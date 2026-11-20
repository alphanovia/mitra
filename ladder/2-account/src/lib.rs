//! Rung 2 of the ladder: a bank account with an overdraft limit. The
//! model is in `model/`.

// mitra: implements bank::Account
#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    balance: i64,
    limit: u64,
    closed: bool,
}

impl Account {
    // mitra: implements bank::Account::new
    pub fn new(limit: u64) -> Account {
        Account { balance: 0, limit, closed: false }
    }

    // mitra: implements bank::Account::deposit
    pub fn deposit(&self, n: u64) -> Account {
        if self.closed {
            return self.clone();
        }
        Account { balance: self.balance + n as i64, ..self.clone() }
    }

    // mitra: implements bank::Account::withdraw
    pub fn withdraw(&self, n: u64) -> Option<Account> {
        if self.closed || self.balance + (self.limit as i64) < n as i64 {
            return None;
        }
        Some(Account { balance: self.balance - n as i64, ..self.clone() })
    }

    // mitra: implements bank::Account::close
    pub fn close(&self) -> Account {
        Account { closed: true, ..self.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Accounts spread over balances from -limit up, limits and amounts
    /// under a thousand: 10 x 10 x 30 = 3000 cases.
    fn cases(closed: bool) -> Vec<(Account, u64)> {
        let mut all = Vec::new();
        for i in 0..10u64 {
            for j in 0..10u64 {
                for k in 0..30u64 {
                    let limit = i * 111 % 1000;
                    let balance = (j * 397 % 1000) as i64 - limit as i64;
                    all.push((Account { balance, limit, closed }, k * 7919 % 1000));
                }
            }
        }
        all
    }

    // mitra: evidence for bank::starts_empty given limit in small sampled 1000 written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn a_new_account_is_empty() {
        for limit in 0..1000 {
            assert_eq!(Account::new(limit), Account { balance: 0, limit, closed: false });
        }
    }

    // mitra: evidence for bank::deposit_adds given n in small sampled 3000 written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn a_deposit_adds_its_amount() {
        for (a, n) in cases(false) {
            let after = a.deposit(n);
            assert_eq!(after, Account { balance: a.balance + n as i64, ..a });
        }
    }

    // mitra: evidence for bank::withdraw_takes given n in small sampled 3000 written "agent:anthropic/fable-5.2/ladder"
    // mitra: evidence for bank::refuses_past_limit given n in small sampled 3000 written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn a_withdrawal_takes_its_amount_or_is_refused() {
        for (a, n) in cases(false) {
            if a.balance + a.limit as i64 >= n as i64 {
                assert_eq!(a.withdraw(n), Some(Account { balance: a.balance - n as i64, ..a }));
            } else {
                assert_eq!(a.withdraw(n), None);
            }
        }
    }

    // mitra: evidence for bank::closes sampled 3000 written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn closing_only_closes() {
        for (a, _) in cases(false) {
            assert_eq!(a.close(), Account { closed: true, ..a.clone() });
        }
    }

    // mitra: evidence for bank::closed_is_frozen given n in small sampled 3000 written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn a_closed_account_is_frozen() {
        for (a, n) in cases(true) {
            assert_eq!(a.deposit(n), a);
            assert_eq!(a.withdraw(n), None);
        }
    }

    // One exact case of each, as outputs the model can read.
    // mitra: evidence for bank::Account::new given limit = 50 gives Account(0, 50, false) written "agent:anthropic/fable-5.2/ladder"
    // mitra: evidence for bank::Account::deposit given self = Account(0, 50, false), n = 20 gives Account(20, 50, false) written "agent:anthropic/fable-5.2/ladder"
    // mitra: evidence for bank::Account::withdraw given self = Account(20, 50, false), n = 70 gives Some(Account(-50, 50, false)) written "agent:anthropic/fable-5.2/ladder"
    // mitra: evidence for bank::Account::withdraw given self = Account(-50, 50, false), n = 1 gives None written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn down_to_the_limit_and_no_further() {
        let a = Account::new(50);
        assert_eq!(a, Account { balance: 0, limit: 50, closed: false });
        let a = a.deposit(20);
        assert_eq!(a.balance, 20);
        let a = a.withdraw(70).unwrap();
        assert_eq!(a.balance, -50);
        assert_eq!(a.withdraw(1), None);
    }
}
