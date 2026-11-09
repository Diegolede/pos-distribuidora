import type { Product, Cart, Sale, MonthlySummary } from './types';

declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
      };
    };
  }
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (window.__TAURI__) {
    return window.__TAURI__.core.invoke<T>(cmd, args);
  }
  throw new Error('Tauri no está disponible');
}

export const api = {
  async getProducts(limit?: number): Promise<Product[]> {
    return invoke('get_products', { limit });
  },

  async searchProducts(query: string, category?: string, limit?: number): Promise<Product[]> {
    return invoke('search_products', { query, category, limit });
  },

  async getCategories(): Promise<string[]> {
    return invoke('get_categories');
  },

  async generateTestData(count: number): Promise<number> {
    return invoke('generate_test_data', { count });
  },

  async addToCart(productId: number, quantity: number): Promise<Cart> {
    return invoke('add_to_cart', { productId, quantity });
  },

  async updateCartQuantity(productId: number, quantity: number): Promise<Cart> {
    return invoke('update_cart_quantity', { productId, quantity });
  },

  async removeFromCart(productId: number): Promise<Cart> {
    return invoke('remove_from_cart', { productId });
  },

  async setDefaultPriceType(priceType: string): Promise<Cart> {
    return invoke('set_default_price_type', { priceType });
  },

  async getCart(): Promise<Cart> {
    return invoke('get_cart');
  },

  async clearCart(): Promise<void> {
    return invoke('clear_cart');
  },

  async createSale(paymentMethod: string, cashReceivedPesos?: number): Promise<Sale> {
    return invoke('create_sale', { paymentMethod, cashReceivedPesos });
  },

  async getMonthlySummary(year: number, month: number): Promise<MonthlySummary> {
    return invoke('get_monthly_summary', { year, month });
  },

  async getSalesForMonth(year: number, month: number): Promise<Sale[]> {
    return invoke('get_sales_for_month', { year, month });
  },

  async voidSale(saleId: number, reason: string): Promise<void> {
    return invoke('void_sale', { saleId, reason });
  },

  async setItemPriceType(productId: number, priceType: string): Promise<void> {
    return invoke('set_item_price_type', { productId, priceType });
  },

  async resetAllToDefault(): Promise<Cart> {
    return invoke('reset_all_to_default');
  },
};
