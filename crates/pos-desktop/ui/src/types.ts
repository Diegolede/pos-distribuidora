export interface Product {
  id: number;
  name: string;
  description: string | null;
  price_professional: Money | null;
  price_public: Money | null;
  price_professional_calculated: boolean;
  price_public_calculated: boolean;
  category: string | null;
  sku: string | null;
  created_at: string;
  updated_at: string;
}

export interface Money {
  amount: string;
  currency: string;
}

export interface CartItem {
  product_id: number;
  product_name: string;
  quantity: number;
  price_type: PriceType;
  unit_price: Money;
}

export interface Cart {
  items: CartItem[];
  default_price_type: PriceType;
}

export type PriceType = 'Professional' | 'Public' | 'Inherit';

export interface Sale {
  id: number;
  sale_number: string;
  timestamp: string;
  created_at: string;
  items: SaleItem[];
  total: Money;
  payment_method: PaymentMethod;
  cash_received: Money | null;
  cash_change: Money | null;
  is_voided: boolean;
  voided_at: string | null;
  void_reason: string | null;
}

export interface SaleItem {
  product_id: number;
  product_name: string;
  quantity: number;
  unit_price: Money;
  price_type: PriceType;
  subtotal: Money;
}

export type PaymentMethod = 'Efectivo' | 'Tarjeta' | 'Transferencia';

export interface MonthlySummary {
  year: number;
  month: number;
  transaction_count: number;
  total_sales: Money;
  total_voided: Money;
  net_sales: Money;
  voided_count: number;
  units_sold: number;
  cash_total: Money;
  card_total: Money;
  transfer_total: Money;
}

export interface ImportPreview {
  total_rows: number;
  valid_rows: number;
  new_products: number;
  updates: number;
  errors: ImportError[];
  calculated_professional_prices: number;
  calculated_public_prices: number;
  products_without_prices: number;
}

export interface ImportError {
  row: number;
  column: string | null;
  message: string;
}
