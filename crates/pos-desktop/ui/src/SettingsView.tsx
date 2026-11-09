import { useState } from 'react';

export function SettingsView() {
  const [companyName, setCompanyName] = useState('Mi Distribuidora');
  const [address, setAddress] = useState('');
  const [phone, setPhone] = useState('');
  const [priceMarkup, setPriceMarkup] = useState('30');
  const [currency, setCurrency] = useState('ARS');
  const [timezone, setTimezone] = useState('America/Argentina/Buenos_Aires');
  const [printerType, setPrinterType] = useState('58mm');

  const handleSave = () => {
    // TODO: Guardar en backend con Tauri
    alert('Configuración guardada correctamente');
  };

  return (
    <div className="settings-container">
      <div className="settings-header">
        <h2 className="settings-title">Ajustes</h2>
        <p className="settings-subtitle">
          Configuración de la empresa, precios y preferencias
        </p>
      </div>

      <div className="settings-sections">
        {/* Información de la empresa */}
        <div className="settings-card">
          <h3 className="section-title">Información de la Empresa</h3>
          <div className="settings-form">
            <div className="form-group">
              <label className="form-label">Nombre comercial</label>
              <input
                type="text"
                value={companyName}
                onChange={(e) => setCompanyName(e.target.value)}
                className="input"
                placeholder="Ej: Distribuidora ABC"
              />
            </div>
            <div className="form-group">
              <label className="form-label">Dirección</label>
              <input
                type="text"
                value={address}
                onChange={(e) => setAddress(e.target.value)}
                className="input"
                placeholder="Calle 123, Ciudad"
              />
            </div>
            <div className="form-group">
              <label className="form-label">Teléfono</label>
              <input
                type="text"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                className="input"
                placeholder="+54 11 1234-5678"
              />
            </div>
          </div>
        </div>

        {/* Configuración de precios */}
        <div className="settings-card">
          <h3 className="section-title">Configuración de Precios</h3>
          <div className="settings-form">
            <div className="form-group">
              <label className="form-label">Recargo por defecto (%)</label>
              <input
                type="number"
                value={priceMarkup}
                onChange={(e) => setPriceMarkup(e.target.value)}
                className="input"
                min="0"
                max="100"
                step="1"
              />
              <p className="help-text">
                Markup aplicado para calcular automáticamente el precio público a partir del profesional 
                (o viceversa) cuando falta un precio en la importación.
                <br />
                Ejemplo: Con 30%, un precio profesional de $100 calcula un público de $130.
              </p>
            </div>
            <div className="form-group">
              <label className="form-label">Moneda</label>
              <select
                value={currency}
                onChange={(e) => setCurrency(e.target.value)}
                className="input"
              >
                <option value="ARS">Peso Argentino (ARS)</option>
                <option value="USD">Dólar (USD)</option>
              </select>
            </div>
          </div>
        </div>

        {/* Impresora */}
        <div className="settings-card">
          <h3 className="section-title">Configuración de Impresora</h3>
          <div className="settings-form">
            <div className="form-group">
              <label className="form-label">Ancho de ticket</label>
              <div className="segmented-control">
                <button
                  onClick={() => setPrinterType('58mm')}
                  className={printerType === '58mm' ? 'segment segment-active' : 'segment'}
                >
                  58 mm
                </button>
                <button
                  onClick={() => setPrinterType('80mm')}
                  className={printerType === '80mm' ? 'segment segment-active' : 'segment'}
                >
                  80 mm
                </button>
              </div>
            </div>
            <div className="form-group">
              <label className="form-label">Nombre de impresora</label>
              <input
                type="text"
                className="input"
                placeholder="Ej: Epson TM-T20"
                disabled
              />
              <p className="help-text">
                Detección automática de impresoras (próximamente)
              </p>
            </div>
          </div>
        </div>

        {/* Regional */}
        <div className="settings-card">
          <h3 className="section-title">Configuración Regional</h3>
          <div className="settings-form">
            <div className="form-group">
              <label className="form-label">Zona horaria</label>
              <select
                value={timezone}
                onChange={(e) => setTimezone(e.target.value)}
                className="input"
              >
                <option value="America/Argentina/Buenos_Aires">Buenos Aires (UTC-3)</option>
                <option value="America/Argentina/Cordoba">Córdoba (UTC-3)</option>
                <option value="America/Argentina/Mendoza">Mendoza (UTC-3)</option>
              </select>
              <p className="help-text">
                Usado para reportes y timestamps de ventas
              </p>
            </div>
          </div>
        </div>

        {/* Atajos de teclado */}
        <div className="settings-card">
          <h3 className="section-title">Atajos de Teclado</h3>
          <div className="keyboard-shortcuts">
            <div className="shortcut-item">
              <kbd>F2</kbd>
              <span>Focus en buscador</span>
            </div>
            <div className="shortcut-item">
              <kbd>F12</kbd>
              <span>Cobrar</span>
            </div>
            <div className="shortcut-item">
              <kbd>Alt</kbd> + <kbd>1</kbd>
              <span>Ver todas las categorías</span>
            </div>
            <div className="shortcut-item">
              <kbd>Alt</kbd> + <kbd>2-9</kbd>
              <span>Seleccionar categoría específica</span>
            </div>
            <div className="shortcut-item">
              <kbd>Enter</kbd>
              <span>Confirmar venta (en modal de pago)</span>
            </div>
          </div>
        </div>
      </div>

      <div className="settings-actions">
        <button onClick={handleSave} className="btn btn-primary btn-large">
          Guardar Cambios
        </button>
      </div>
    </div>
  );
}
