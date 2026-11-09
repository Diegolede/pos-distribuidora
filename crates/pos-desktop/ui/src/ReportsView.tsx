import { useState, useEffect } from 'react';
import { api } from './api';
import type { Sale, MonthlySummary } from './types';
import { formatMoney } from './utils';

export function ReportsView() {
  const now = new Date();
  const [year, setYear] = useState(now.getFullYear());
  const [month, setMonth] = useState(now.getMonth() + 1);
  const [summary, setSummary] = useState<MonthlySummary | null>(null);
  const [sales, setSales] = useState<Sale[]>([]);
  const [loading, setLoading] = useState(false);
  const [voidingId, setVoidingId] = useState<number | null>(null);
  const [voidReason, setVoidReason] = useState('');

  useEffect(() => {
    loadReports();
  }, [year, month]);

  const loadReports = async () => {
    try {
      setLoading(true);
      const [summaryData, salesData] = await Promise.all([
        api.getMonthlySummary(year, month),
        api.getSalesForMonth(year, month),
      ]);
      setSummary(summaryData);
      setSales(salesData);
    } catch (error) {
      console.error('Error al cargar reportes:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleVoidSale = async (saleId: number) => {
    if (!voidReason.trim()) {
      alert('Debe ingresar un motivo para anular la venta');
      return;
    }
    
    try {
      await api.voidSale(saleId, voidReason);
      setVoidingId(null);
      setVoidReason('');
      await loadReports();
    } catch (error) {
      alert('Error al anular venta: ' + error);
    }
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString('es-AR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const exportToCSV = () => {
    const headers = ['ID', 'Fecha', 'Total', 'Método de Pago', 'Estado'];
    const rows = sales.map(s => [
      s.id,
      formatDate(s.created_at),
      s.total.amount,
      s.payment_method,
      s.voided_at ? 'ANULADA' : 'COMPLETADA'
    ]);
    
    const csv = [headers, ...rows].map(row => row.join(',')).join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `ventas-${year}-${month.toString().padStart(2, '0')}.csv`;
    a.click();
  };

  return (
    <div className="reports-container">
      <div className="reports-header">
        <h2 className="reports-title">Reportes de Ventas</h2>
        
        <div className="date-selector">
          <select
            value={month}
            onChange={(e) => setMonth(parseInt(e.target.value))}
            className="input month-select"
          >
            {Array.from({ length: 12 }, (_, i) => i + 1).map(m => (
              <option key={m} value={m}>
                {new Date(2000, m - 1).toLocaleString('es-AR', { month: 'long' })}
              </option>
            ))}
          </select>
          <input
            type="number"
            value={year}
            onChange={(e) => setYear(parseInt(e.target.value))}
            className="input year-input"
            min="2020"
            max="2100"
          />
          <button onClick={loadReports} className="btn btn-secondary">
            Actualizar
          </button>
        </div>
      </div>

      {loading ? (
        <div className="loading-spinner">
          <div className="spinner"></div>
          <p>Cargando reportes...</p>
        </div>
      ) : summary ? (
        <>
          <div className="summary-cards">
            <div className="summary-card">
              <span className="summary-card-label">Total Bruto</span>
              <span className="summary-card-value">{formatMoney(summary.total_sales)}</span>
              <span className="summary-card-detail">{summary.transaction_count} transacciones</span>
            </div>
            
            <div className="summary-card">
              <span className="summary-card-label">Anulaciones</span>
              <span className="summary-card-value negative">{formatMoney(summary.total_voided)}</span>
              <span className="summary-card-detail">{summary.voided_count} anuladas</span>
            </div>
            
            <div className="summary-card highlight">
              <span className="summary-card-label">Total Neto</span>
              <span className="summary-card-value">{formatMoney(summary.net_sales)}</span>
              <span className="summary-card-detail">
                {summary.units_sold} unidades vendidas
              </span>
            </div>
          </div>

          <div className="payment-methods-breakdown">
            <h3>Desglose por Método de Pago</h3>
            <div className="payment-grid">
              <div className="payment-item">
                <span className="payment-label">💵 Efectivo</span>
                <span className="payment-amount">{formatMoney(summary.cash_total)}</span>
              </div>
              <div className="payment-item">
                <span className="payment-label">💳 Tarjeta</span>
                <span className="payment-amount">{formatMoney(summary.card_total)}</span>
              </div>
              <div className="payment-item">
                <span className="payment-label">🏦 Transferencia</span>
                <span className="payment-amount">{formatMoney(summary.transfer_total)}</span>
              </div>
            </div>
          </div>

          <div className="sales-table-section">
            <div className="table-header">
              <h3>Detalle de Ventas</h3>
              <button onClick={exportToCSV} className="btn btn-secondary">
                Exportar CSV
              </button>
            </div>
            
            <div className="table-container">
              <table className="sales-table">
                <thead>
                  <tr>
                    <th>ID</th>
                    <th>Fecha</th>
                    <th>Total</th>
                    <th>Método</th>
                    <th>Estado</th>
                    <th>Acciones</th>
                  </tr>
                </thead>
                <tbody>
                  {sales.map(sale => (
                    <tr key={sale.id} className={sale.voided_at ? 'voided-row' : ''}>
                      <td>#{sale.id}</td>
                      <td>{formatDate(sale.created_at)}</td>
                      <td>{formatMoney(sale.total)}</td>
                      <td>{sale.payment_method}</td>
                      <td>
                        {sale.voided_at ? (
                          <span className="status-badge voided">ANULADA</span>
                        ) : (
                          <span className="status-badge completed">COMPLETADA</span>
                        )}
                      </td>
                      <td>
                        {!sale.voided_at && (
                          <button
                            onClick={() => setVoidingId(sale.id)}
                            className="btn btn-danger btn-small"
                          >
                            Anular
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </>
      ) : (
        <div className="empty-state">
          <p>No hay ventas para este período</p>
        </div>
      )}

      {voidingId && (
        <div className="modal-overlay" onClick={() => setVoidingId(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3>Anular Venta #{voidingId}</h3>
            </div>
            <div className="modal-body">
              <label className="form-label">Motivo de anulación:</label>
              <textarea
                className="input"
                rows={4}
                value={voidReason}
                onChange={(e) => setVoidReason(e.target.value)}
                placeholder="Escriba el motivo de la anulación..."
                autoFocus
              />
            </div>
            <div className="modal-footer">
              <button onClick={() => setVoidingId(null)} className="btn btn-secondary">
                Cancelar
              </button>
              <button
                onClick={() => handleVoidSale(voidingId)}
                disabled={!voidReason.trim()}
                className="btn btn-danger"
              >
                Confirmar Anulación
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
