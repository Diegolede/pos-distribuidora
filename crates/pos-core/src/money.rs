use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    ARS, // Peso argentino
    USD,
}

impl Currency {
    pub fn symbol(&self) -> &str {
        match self {
            Currency::ARS => "$",
            Currency::USD => "US$",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    amount: Decimal,
    currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn from_cents(cents: i64, currency: Currency) -> Self {
        Self {
            amount: Decimal::new(cents, 2),
            currency,
        }
    }

    pub fn from_pesos(pesos: i64) -> Self {
        Self::from_cents(pesos * 100, Currency::ARS)
    }

    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn to_cents(&self) -> i64 {
        let cents = (self.amount * Decimal::from(100)).round();
        cents.to_string().parse::<f64>().unwrap_or(0.0) as i64
    }

    /// Formato argentino: $ 1.234,56
    pub fn format_ar(&self) -> String {
        let integer_part = self.amount.trunc().abs();
        let fractional_dec = (self.amount.fract().abs() * Decimal::from(100)).round();
        let fractional = fractional_dec.to_string().parse::<f64>().unwrap_or(0.0) as i64;

        let sign = if self.amount.is_sign_negative() {
            "-"
        } else {
            ""
        };

        let integer_str = integer_part.to_string();
        let mut formatted = String::new();
        
        for (i, c) in integer_str.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                formatted.push('.');
            }
            formatted.push(c);
        }
        
        let formatted: String = formatted.chars().rev().collect();

        format!(
            "{}{} {},{}",
            sign,
            self.currency.symbol(),
            formatted,
            format!("{:02}", fractional)
        )
    }

    pub fn checked_add(self, other: Money) -> Option<Money> {
        if self.currency != other.currency {
            return None;
        }
        Some(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }

    pub fn checked_sub(self, other: Money) -> Option<Money> {
        if self.currency != other.currency {
            return None;
        }
        Some(Money {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }

    pub fn checked_mul(self, quantity: u32) -> Option<Money> {
        Some(Money {
            amount: self.amount * Decimal::from(quantity),
            currency: self.currency,
        })
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, other: Money) -> Money {
        self.checked_add(other)
            .expect("No se puede sumar dinero de diferentes monedas")
    }
}

impl Sub for Money {
    type Output = Money;

    fn sub(self, other: Money) -> Money {
        self.checked_sub(other)
            .expect("No se puede restar dinero de diferentes monedas")
    }
}

impl Mul<u32> for Money {
    type Output = Money;

    fn mul(self, quantity: u32) -> Money {
        self.checked_mul(quantity)
            .expect("Desbordamiento al multiplicar dinero")
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_ar())
    }
}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.currency != other.currency {
            None
        } else {
            self.amount.partial_cmp(&other.amount)
        }
    }
}

impl Ord for Money {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_money_creation() {
        let money = Money::from_pesos(1234);
        assert_eq!(money.to_cents(), 123400);
    }

    #[test]
    fn test_money_formatting() {
        let money = Money::new(dec!(1234.56), Currency::ARS);
        assert_eq!(money.format_ar(), "$ 1.234,56");

        let money2 = Money::new(dec!(1000000.99), Currency::ARS);
        assert_eq!(money2.format_ar(), "$ 1.000.000,99");
    }

    #[test]
    fn test_money_operations() {
        let m1 = Money::from_pesos(100);
        let m2 = Money::from_pesos(50);
        
        let sum = m1 + m2;
        assert_eq!(sum.to_cents(), 15000);

        let diff = m1 - m2;
        assert_eq!(diff.to_cents(), 5000);

        let product = m1 * 3;
        assert_eq!(product.to_cents(), 30000);
    }

    #[test]
    fn test_money_precision() {
        let m1 = Money::new(dec!(10.33), Currency::ARS);
        let m2 = Money::new(dec!(10.33), Currency::ARS);
        let m3 = Money::new(dec!(10.34), Currency::ARS);
        
        let sum = m1 + m2 + m3;
        assert_eq!(sum.amount(), dec!(31.00));
    }
}
