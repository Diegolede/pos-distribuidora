import { useState, useEffect, useRef } from 'react';
import { 
  Search, ShoppingCart, Plus, Minus, CreditCard, 
  Banknote, Building2, Receipt, Clock 
} from 'lucide-react';
import { api } from './api';
import { formatMoney } from './utils';
import type { Product, Cart } from './types';
import { ImportView } from './ImportView';
import { SettingsView } from './SettingsView';
import { ReportsView } from './ReportsView';
import './App.css';

type PriceType = 'Public' | 'Professional' | 'Inherit';
type PaymentMethod = 'Efectivo' | 'Tarjeta' | 'Transferencia';
type View = 'sale' | 'reports' | 'import' | 'settings';

function App() {
  const [view, setView] = useState<View>('sale');
  const [products, setProducts] = useState<Product[]>([]);
  const [allProducts, setAllProducts] = useState<Product[]>([]);
  const [cart, setCart] = useState<Cart | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [categories, setCategories] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [showPaymentModal, setShowPaymentModal] = useState(false);
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>('Efectivo');
  const [cashReceived, setCashReceived] = useState('');
  const [currentTime, setCurrentTime] = useState(new Date());
  const searchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    loadProducts();
    loadCart();
    loadCategories();
    
    const timer = setInterval(() => setCurrentTime(new Date()), 1000);
    
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'F2') {
        e.preventDefault();
        searchInputRef.current?.focus();
      } else if (e.key === 'F12' && view === 'sale') {
        e.preventDefault();
        openPaymentModal();
      } else if (e.altKey && view === 'sale' && document.activeElement !== searchInputRef.current) {
        const num = parseInt(e.key);
        if (num >= 1 && num <= 9) {
          e.preventDefault();
          if (num === 1) {
            setSelectedCategory(null);
          } else if (categories[num - 2]) {
            setSelectedCategory(categories[num - 2]);
          }
        }
      } else if (e.key >= '1' && e.key <= '3' && showPaymentModal) {
        e.preventDefault();
        const methods: PaymentMethod[] = ['Efectivo', 'Tarjeta', 'Transferencia'];
        setPaymentMethod(methods[parseInt(e.key) - 1]);
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      clearInterval(timer);
    };
  }, [view, categories, showPaymentModal]);

  useEffect(() => {
    const delayDebounceFn = setTimeout(() => {
      handleSearch();
    }, 300);
    return () => clearTimeout(delayDebounceFn);
  }, [searchQuery, selectedCategory]);

  const loadProducts = async () => {
    try {
      setLoading(true);
      const prods = await api.getProducts(5000);
      setAllProducts(prods);
      setProducts(prods);
    } catch (error) {
      console.error('Error al cargar productos:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadCategories = async () => {
    try {
      const cats = await api.getCategories();
      setCategories(cats);
    } catch (error) {
      console.error('Error al cargar categorías:', error);
    }
  };

  const loadCart = async () => {
    try {
      const c = await api.getCart();
      setCart(c);
    } catch (error) {
      console.error('Error al cargar carrito:', error);
    }
  };

  const handleSearch = async () => {
    try {
      setLoading(true);
      if (!searchQuery.trim() && !selectedCategory) {
        setProducts(allProducts);
        return;
      }
      
      const results = await api.searchProducts(
        searchQuery.trim(),
        selectedCategory || undefined,
        500
      );
      setProducts(results);
    } catch (error) {
      console.error('Error en búsqueda:', error);
    } finally {
      setLoading(false);
    }
  };

  const addToCart = async (productId: number) => {
    try {
      await api.addToCart(productId, 1);
      await loadCart();
    } catch (error) {
      alert('Error al agregar al carrito: ' + error);
    }
  };

  const updateQuantity = async (productId: number, quantity: number) => {
    if (quantity <= 0) {
      try {
        await api.removeFromCart(productId);
        await loadCart();
      } catch (error) {
        alert('Error al eliminar del carrito: ' + error);
      }
    } else {
      try {
        await api.updateCartQuantity(productId, quantity);
        await loadCart();
      } catch (error) {
        alert('Error al actualizar cantidad: ' + error);
      }
    }
  };

  const setItemPriceType = async (productId: number, priceType: PriceType) => {
    try {
      await api.setItemPriceType(productId, priceType);
      await loadCart();
    } catch (error) {
      alert('Error al cambiar tipo de precio: ' + error);
    }
  };

  const setDefaultPriceType = async (priceType: PriceType) => {
    try {
      await api.setDefaultPriceType(priceType);
      await loadCart();
    } catch (error) {
      alert('Error al cambiar precio general: ' + error);
    }
  };

  const openPaymentModal = () => {
    if (!cart || cart.items.length === 0) return;
    setShowPaymentModal(true);
    setCashReceived('');
  };

  const finalizeSale = async () => {
    try {
      await api.createSale(paymentMethod);
      setShowPaymentModal(false);
      await loadCart();
      setCashReceived('');
      alert('Venta completada exitosamente');
    } catch (error) {
      alert('Error al completar venta: ' + error);
    }
  };

  const cartTotal = cart?.items.reduce((sum, item) => {
    const price = parseFloat(item.unit_price.amount) / 100;
    return sum + (price * item.quantity);
  }, 0) || 0;

  const cartItemCount = cart?.items.reduce((sum, item) => sum + item.quantity, 0) || 0;

  const cashReceivedNum = parseFloat(cashReceived.replace(',', '.') || '0');
  const changeAmount = cashReceivedNum - cartTotal;

  return (
    <div className="app-container">
      {/* Header */}
      <header className="header">
        <div className="header-logo">POS Distribuidora</div>
        <nav className="header-nav">
          <button
            className={`nav-btn ${view === 'sale' ? 'active' : ''}`}
            onClick={() => setView('sale')}
          >
            Ventas
          </button>
          <button
            className={`nav-btn ${view === 'reports' ? 'active' : ''}`}
            onClick={() => setView('reports')}
          >
            Reportes
          </button>
          <button
            className={`nav-btn ${view === 'import' ? 'active' : ''}`}
            onClick={() => setView('import')}
          >
            Importar
          </button>
          <button
            className={`nav-btn ${view === 'settings' ? 'active' : ''}`}
            onClick={() => setView('settings')}
          >
            Ajustes
          </button>
        </nav>
        <div className="header-spacer" />
        <div className="header-time">
          <Clock size={14} style={{ display: 'inline-block', marginRight: '6px', verticalAlign: 'middle' }} />
          {currentTime.toLocaleTimeString('es-AR', { hour: '2-digit', minute: '2-digit' })}
        </div>
      </header>

      {/* Main Content */}
      {view === 'sale' ? (
        <main className="main-content">
          {/* Products Section */}
          <section className="products-section">
            {/* Search */}
            <div className="search-bar">
              <div className="search-input-wrapper">
                <Search className="search-icon" size={20} />
                <input
                  ref={searchInputRef}
                  type="text"
                  className="search-input"
                  placeholder="Buscar productos..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  autoFocus
                />
                <span className="search-kbd">F2</span>
              </div>
            </div>

            {/* Categories */}
            <div className="categories-bar">
              <div className="categories-list">
                <button
                  className={`category-pill ${!selectedCategory ? 'active' : ''}`}
                  onClick={() => setSelectedCategory(null)}
                >
                  Todas
                </button>
                {categories.map((cat) => (
                  <button
                    key={cat}
                    className={`category-pill ${selectedCategory === cat ? 'active' : ''}`}
                    onClick={() => setSelectedCategory(cat)}
                  >
                    {cat}
                  </button>
                ))}
              </div>
            </div>

            {/* Products Grid */}
            <div className="products-grid-wrapper">
              {loading ? (
                <div className="loading-state">
                  <div className="spinner" />
                </div>
              ) : products.length === 0 ? (
                <div className="empty-state">
                  <Search className="empty-icon" size={48} />
                  <div className="empty-title">Sin resultados</div>
                  <div className="empty-description">
                    No se encontraron productos con los filtros aplicados
                  </div>
                </div>
              ) : (
                <div className="products-grid">
                  {products.map((product) => {
                    const notSellable = !product.price_public && !product.price_professional;
                    return (
                      <div
                        key={product.id}
                        className={`product-card ${notSellable ? 'not-sellable' : ''}`}
                        onClick={() => !notSellable && addToCart(product.id)}
                      >
                        {(product.price_professional_calculated || product.price_public_calculated) && (
                          <div className="product-calc-badge">*</div>
                        )}
                        <div className="product-name">{product.name}</div>
                        {product.category && (
                          <div className="product-category">{product.category}</div>
                        )}
                        {!notSellable && (
                          <div className="product-prices">
                            <div className="product-price-main">
                              {formatMoney(product.price_public!)}
                            </div>
                            {product.price_professional && (
                              <div className="product-price-secondary">
                                Prof. {formatMoney(product.price_professional)}
                              </div>
                            )}
                          </div>
                        )}
                        {notSellable && (
                          <div className="product-category">Sin precio</div>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          </section>

          {/* Cart Panel */}
          <aside className="cart-panel">
            <div className="cart-header">
              <div className="cart-title">
                <ShoppingCart size={16} style={{ display: 'inline-block', marginRight: '6px', verticalAlign: 'middle' }} />
                Carrito
              </div>
              <div className="cart-count">{cartItemCount} items</div>
            </div>

            {cart && cart.items.length > 0 && (
              <div className="cart-price-selector">
                <div className="price-selector-label">Precio General</div>
                <div className="segmented-control">
                  <button
                    className={`segment ${cart.default_price_type === 'Public' ? 'active' : ''}`}
                    onClick={() => setDefaultPriceType('Public')}
                  >
                    Público
                  </button>
                  <button
                    className={`segment ${cart.default_price_type === 'Professional' ? 'active' : ''}`}
                    onClick={() => setDefaultPriceType('Professional')}
                  >
                    Profesional
                  </button>
                </div>
              </div>
            )}

            <div className="cart-items">
              {!cart || cart.items.length === 0 ? (
                <div className="empty-state">
                  <ShoppingCart className="empty-icon" size={48} />
                  <div className="empty-title">Carrito vacío</div>
                  <div className="empty-description">
                    Agregá productos para comenzar una venta
                  </div>
                </div>
              ) : (
                cart.items.map((item) => (
                  <div key={item.product_id} className="cart-item">
                    <div className="cart-item-header">
                      <div className="cart-item-name">{item.product_name}</div>
                      <div className="cart-item-subtotal">
                        {formatMoney({
                          amount: (parseFloat(item.unit_price.amount) * item.quantity).toString(),
                          currency: 'ARS'
                        })}
                      </div>
                    </div>
                    <div className="cart-item-controls">
                      <div className="quantity-control">
                        <button
                          className="qty-btn"
                          onClick={() => updateQuantity(item.product_id, item.quantity - 1)}
                        >
                          <Minus size={14} />
                        </button>
                        <input
                          type="number"
                          className="qty-input"
                          value={item.quantity}
                          onChange={(e) => {
                            const val = parseInt(e.target.value) || 0;
                            if (val >= 0) updateQuantity(item.product_id, val);
                          }}
                          min="0"
                        />
                        <button
                          className="qty-btn"
                          onClick={() => updateQuantity(item.product_id, item.quantity + 1)}
                        >
                          <Plus size={14} />
                        </button>
                      </div>
                      <div className="item-price-switch">
                        <button
                          className={`item-price-btn ${item.price_type === 'Inherit' ? 'active' : ''}`}
                          onClick={() => setItemPriceType(item.product_id, 'Inherit')}
                        >
                          Auto
                        </button>
                        <button
                          className={`item-price-btn ${item.price_type === 'Public' ? 'active' : ''}`}
                          onClick={() => setItemPriceType(item.product_id, 'Public')}
                        >
                          Púb
                        </button>
                        <button
                          className={`item-price-btn ${item.price_type === 'Professional' ? 'active' : ''}`}
                          onClick={() => setItemPriceType(item.product_id, 'Professional')}
                        >
                          Prof
                        </button>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>

            {cart && cart.items.length > 0 && (
              <div className="cart-footer">
                <div className="cart-summary">
                  <div className="summary-row">
                    <span className="summary-label">Subtotal</span>
                    <span className="summary-value">
                      $ {cartTotal.toFixed(2).replace('.', ',').replace(/\B(?=(\d{3})+(?!\d))/g, '.')}
                    </span>
                  </div>
                  <div className="summary-row summary-total">
                    <span className="summary-label">Total</span>
                    <span className="summary-value">
                      $ {cartTotal.toFixed(2).replace('.', ',').replace(/\B(?=(\d{3})+(?!\d))/g, '.')}
                    </span>
                  </div>
                </div>
                <button className="btn-checkout" onClick={openPaymentModal}>
                  <Receipt size={20} />
                  Cobrar
                  <span className="btn-kbd">F12</span>
                </button>
              </div>
            )}
          </aside>
        </main>
      ) : view === 'reports' ? (
        <ReportsView />
      ) : view === 'import' ? (
        <ImportView />
      ) : (
        <SettingsView />
      )}

      {/* Payment Modal */}
      {showPaymentModal && (
        <div className="modal-overlay" onClick={() => setShowPaymentModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3 className="modal-title">Cobrar</h3>
            </div>
            <div className="modal-body">
              <div className="payment-amount">
                <div className="payment-label">Total a cobrar</div>
                <div className="payment-total">
                  $ {cartTotal.toFixed(2).replace('.', ',').replace(/\B(?=(\d{3})+(?!\d))/g, '.')}
                </div>
              </div>

              <div>
                <div className="payment-label" style={{ marginBottom: '12px' }}>Método de pago</div>
                <div className="payment-methods">
                  <button
                    className={`payment-method-btn ${paymentMethod === 'Efectivo' ? 'active' : ''}`}
                    onClick={() => setPaymentMethod('Efectivo')}
                  >
                    <Banknote size={24} />
                    <span className="payment-method-label">Efectivo</span>
                    <span className="payment-method-kbd">1</span>
                  </button>
                  <button
                    className={`payment-method-btn ${paymentMethod === 'Tarjeta' ? 'active' : ''}`}
                    onClick={() => setPaymentMethod('Tarjeta')}
                  >
                    <CreditCard size={24} />
                    <span className="payment-method-label">Tarjeta</span>
                    <span className="payment-method-kbd">2</span>
                  </button>
                  <button
                    className={`payment-method-btn ${paymentMethod === 'Transferencia' ? 'active' : ''}`}
                    onClick={() => setPaymentMethod('Transferencia')}
                  >
                    <Building2 size={24} />
                    <span className="payment-method-label">Transferencia</span>
                    <span className="payment-method-kbd">3</span>
                  </button>
                </div>
              </div>

              {paymentMethod === 'Efectivo' && (
                <div className="cash-input-section">
                  <div>
                    <div className="payment-label" style={{ marginBottom: '8px' }}>Efectivo recibido</div>
                    <div className="quick-amounts">
                      <button
                        className="quick-amount-btn"
                        onClick={() => setCashReceived(cartTotal.toFixed(2))}
                      >
                        Exacto
                      </button>
                      {(() => {
                        const amounts = [];
                        const roundTo1k = Math.ceil(cartTotal / 1000) * 1000;
                        const roundTo5k = Math.ceil(cartTotal / 5000) * 5000;
                        if (roundTo1k > cartTotal) amounts.push(roundTo1k);
                        if (roundTo5k > cartTotal && roundTo5k !== roundTo1k) amounts.push(roundTo5k);
                        return amounts.slice(0, 2).map((amt, idx) => (
                          <button
                            key={idx}
                            className="quick-amount-btn"
                            onClick={() => setCashReceived(amt.toFixed(2))}
                          >
                            $ {amt.toFixed(0).replace(/\B(?=(\d{3})+(?!\d))/g, '.')}
                          </button>
                        ));
                      })()}
                    </div>
                  </div>
                  <input
                    type="text"
                    className="cash-input"
                    placeholder="0,00"
                    value={cashReceived}
                    onChange={(e) => setCashReceived(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && cashReceivedNum >= cartTotal) {
                        finalizeSale();
                      }
                    }}
                    autoFocus
                  />
                  {cashReceived && (
                    <div className={`cash-change ${changeAmount < 0 ? 'insufficient' : ''}`}>
                      <span className="change-label">
                        {changeAmount < 0 ? 'Falta' : 'Vuelto'}
                      </span>
                      <span className="change-amount">
                        $ {Math.abs(changeAmount).toFixed(2).replace('.', ',').replace(/\B(?=(\d{3})+(?!\d))/g, '.')}
                      </span>
                    </div>
                  )}
                </div>
              )}
            </div>
            <div className="modal-footer">
              <button className="btn" onClick={() => setShowPaymentModal(false)}>
                Cancelar
              </button>
              <button
                className="btn-primary"
                onClick={finalizeSale}
                disabled={paymentMethod === 'Efectivo' && cashReceivedNum < cartTotal}
              >
                Confirmar Venta
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
