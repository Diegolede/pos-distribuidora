use chrono::{DateTime, Utc};
use pos_core::{CompanyInfo, Error, Sale};
use std::fmt::Write as FmtWrite;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketWidth {
    Mm58,
    Mm80,
}

impl TicketWidth {
    pub fn chars(&self) -> usize {
        match self {
            TicketWidth::Mm58 => 32,
            TicketWidth::Mm80 => 42,
        }
    }
}

pub struct TicketFormatter {
    width: TicketWidth,
    company: CompanyInfo,
}

impl TicketFormatter {
    pub fn new(width: TicketWidth, company: CompanyInfo) -> Self {
        Self { width, company }
    }

    pub fn format_sale(&self, sale: &Sale) -> Result<String, Error> {
        let mut ticket = String::new();
        let width = self.width.chars();

        self.write_centered(&mut ticket, &"=".repeat(width))?;
        self.write_centered(&mut ticket, &self.company.name)?;
        
        if let Some(ref address) = self.company.address {
            self.write_centered(&mut ticket, address)?;
        }
        if let Some(ref phone) = self.company.phone {
            self.write_centered(&mut ticket, &format!("Tel: {}", phone))?;
        }
        self.write_centered(&mut ticket, &"=".repeat(width))?;

        writeln!(&mut ticket).map_err(|e| Error::Print(e.to_string()))?;

        writeln!(&mut ticket, "Venta Nº: {}", sale.sale_number)
            .map_err(|e| Error::Print(e.to_string()))?;
        writeln!(
            &mut ticket,
            "Fecha: {}",
            sale.timestamp.format("%d/%m/%Y %H:%M:%S")
        )
        .map_err(|e| Error::Print(e.to_string()))?;

        writeln!(&mut ticket).map_err(|e| Error::Print(e.to_string()))?;
        writeln!(&mut ticket, "{}", "-".repeat(width))
            .map_err(|e| Error::Print(e.to_string()))?;

        for item in &sale.items {
            let name_line = if item.product_name.len() > width {
                format!("{}...", &item.product_name[..width - 3])
            } else {
                item.product_name.clone()
            };
            writeln!(&mut ticket, "{}", name_line).map_err(|e| Error::Print(e.to_string()))?;

            let qty_line = format!(
                "{} x {} = {}",
                item.quantity,
                item.unit_price.format_ar(),
                item.subtotal.format_ar()
            );
            writeln!(&mut ticket, "  {}", qty_line).map_err(|e| Error::Print(e.to_string()))?;

            let price_type_line = format!("  ({})", item.price_type.as_str());
            writeln!(&mut ticket, "{}", price_type_line)
                .map_err(|e| Error::Print(e.to_string()))?;
        }

        writeln!(&mut ticket, "{}", "-".repeat(width))
            .map_err(|e| Error::Print(e.to_string()))?;

        let total_line = format!("TOTAL: {}", sale.total.format_ar());
        self.write_right_aligned(&mut ticket, &total_line)?;

        writeln!(&mut ticket).map_err(|e| Error::Print(e.to_string()))?;
        writeln!(&mut ticket, "Pago: {}", sale.payment_method.as_str())
            .map_err(|e| Error::Print(e.to_string()))?;

        if let Some(received) = sale.cash_received {
            writeln!(&mut ticket, "Recibido: {}", received.format_ar())
                .map_err(|e| Error::Print(e.to_string()))?;
        }

        if let Some(change) = sale.cash_change {
            writeln!(&mut ticket, "Cambio: {}", change.format_ar())
                .map_err(|e| Error::Print(e.to_string()))?;
        }

        writeln!(&mut ticket).map_err(|e| Error::Print(e.to_string()))?;
        self.write_centered(&mut ticket, "¡Gracias por su compra!")?;
        writeln!(&mut ticket).map_err(|e| Error::Print(e.to_string()))?;
        self.write_centered(&mut ticket, &"=".repeat(width))?;

        if sale.is_voided {
            writeln!(&mut ticket).map_err(|e| Error::Print(e.to_string()))?;
            self.write_centered(&mut ticket, "*** ANULADA ***")?;
            if let Some(ref reason) = sale.void_reason {
                self.write_centered(&mut ticket, &format!("Motivo: {}", reason))?;
            }
        }

        Ok(ticket)
    }

    fn write_centered(&self, output: &mut String, text: &str) -> Result<(), Error> {
        let width = self.width.chars();
        let text_len = text.chars().count();

        if text_len >= width {
            writeln!(output, "{}", text).map_err(|e| Error::Print(e.to_string()))?;
        } else {
            let padding = (width - text_len) / 2;
            writeln!(output, "{}{}", " ".repeat(padding), text)
                .map_err(|e| Error::Print(e.to_string()))?;
        }

        Ok(())
    }

    fn write_right_aligned(&self, output: &mut String, text: &str) -> Result<(), Error> {
        let width = self.width.chars();
        let text_len = text.chars().count();

        if text_len >= width {
            writeln!(output, "{}", text).map_err(|e| Error::Print(e.to_string()))?;
        } else {
            let padding = width - text_len;
            writeln!(output, "{}{}", " ".repeat(padding), text)
                .map_err(|e| Error::Print(e.to_string()))?;
        }

        Ok(())
    }
}

pub fn save_ticket_to_file(ticket: &str, path: &std::path::Path) -> Result<(), Error> {
    std::fs::write(path, ticket).map_err(|e| Error::Print(format!("Error al guardar ticket: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use pos_core::{Money, PaymentMethod, PriceType, SaleItem};

    #[test]
    fn test_ticket_formatting() {
        let company = CompanyInfo {
            name: "Distribuidora Test".to_string(),
            address: Some("Calle Falsa 123".to_string()),
            phone: Some("011-1234-5678".to_string()),
            ..Default::default()
        };

        let formatter = TicketFormatter::new(TicketWidth::Mm58, company);

        let sale = Sale {
            id: 1,
            sale_number: "202611090001".to_string(),
            timestamp: Utc::now(),
            items: vec![
                SaleItem {
                    product_id: 1,
                    product_name: "Shampoo Test".to_string(),
                    quantity: 2,
                    unit_price: Money::from_pesos(100),
                    price_type: PriceType::Professional,
                    subtotal: Money::from_pesos(200),
                },
                SaleItem {
                    product_id: 2,
                    product_name: "Acondicionador Test".to_string(),
                    quantity: 1,
                    unit_price: Money::from_pesos(150),
                    price_type: PriceType::Public,
                    subtotal: Money::from_pesos(150),
                },
            ],
            total: Money::from_pesos(350),
            payment_method: PaymentMethod::Cash,
            cash_received: Some(Money::from_pesos(400)),
            cash_change: Some(Money::from_pesos(50)),
            is_voided: false,
            void_reason: None,
            void_timestamp: None,
        };

        let ticket = formatter.format_sale(&sale).unwrap();
        assert!(ticket.contains("Distribuidora Test"));
        assert!(ticket.contains("Shampoo Test"));
        assert!(ticket.contains("$ 350,00"));
    }
}
