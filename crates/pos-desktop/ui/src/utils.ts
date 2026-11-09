import type { Money } from './types';

export function formatMoney(money: Money | null): string {
  if (!money) return '$ 0,00';
  
  const amount = parseFloat(money.amount);
  const integerPart = Math.floor(Math.abs(amount));
  const fractionalPart = Math.round((Math.abs(amount) - integerPart) * 100);
  
  const formatted = integerPart.toString().replace(/\B(?=(\d{3})+(?!\d))/g, '.');
  const sign = amount < 0 ? '-' : '';
  
  return `${sign}$ ${formatted},${fractionalPart.toString().padStart(2, '0')}`;
}

export function parseMoney(str: string): number {
  const cleaned = str.replace(/[^\d,.-]/g, '').replace('.', '').replace(',', '.');
  return parseFloat(cleaned) || 0;
}
