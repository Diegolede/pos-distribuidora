import { useState } from 'react';

type ImportStep = 'select' | 'preview' | 'importing' | 'complete';

interface ImportPreview {
  total_rows: number;
  valid_rows: number;
  new_products: number;
  updates: number;
  errors: Array<{
    row: number;
    column: string | null;
    message: string;
  }>;
  calculated_professional_prices: number;
  calculated_public_prices: number;
  products_without_prices: number;
}

export function ImportView() {
  const [step, setStep] = useState<ImportStep>('select');
  const [file, setFile] = useState<File | null>(null);
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ImportPreview | null>(null);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      setFile(e.target.files[0]);
    }
  };

  const handlePreview = async () => {
    if (!file) return;

    setLoading(true);
    try {
      // En producción, esto llamaría a una función Tauri que lee el Excel
      // Por ahora, simulamos la vista previa
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      const mockPreview: ImportPreview = {
        total_rows: 2813,
        valid_rows: 2721,
        new_products: 2673,
        updates: 48,
        errors: [],
        calculated_professional_prices: 156,
        calculated_public_prices: 87,
        products_without_prices: 92
      };
      
      setPreview(mockPreview);
      setStep('preview');
    } catch (error) {
      alert('Error al procesar el archivo: ' + error);
    } finally {
      setLoading(false);
    }
  };

  const handleImport = async () => {
    if (!file) return;

    setLoading(true);
    setStep('importing');
    try {
      // Llamar a la función Tauri de importación
      await new Promise(resolve => setTimeout(resolve, 3000));
      
      setResult(preview);
      setStep('complete');
    } catch (error) {
      alert('Error al importar: ' + error);
      setStep('preview');
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    setStep('select');
    setFile(null);
    setPreview(null);
    setResult(null);
  };

  return (
    <div className="import-container">
      <div className="import-header">
        <h2 className="import-title">Asistente de Importación</h2>
        <p className="import-subtitle">
          Importa productos desde un archivo Excel (.xlsx) con las columnas requeridas
        </p>
      </div>

      {step === 'select' && (
        <div className="import-card">
          <div className="import-step">
            <h3 className="step-title">1. Seleccionar archivo</h3>
            <p className="step-description">
              El archivo debe contener las columnas: Nombre, Precio Profesional, Precio Público, 
              Categoría (opcional), SKU (opcional), Descripción (opcional)
            </p>
            
            <div className="file-input-container">
              <input
                type="file"
                accept=".xlsx,.xls"
                onChange={handleFileSelect}
                className="file-input"
                id="file-upload"
              />
              <label htmlFor="file-upload" className="file-label">
                {file ? file.name : 'Seleccionar archivo Excel...'}
              </label>
            </div>

            <div className="import-actions">
              <button
                onClick={handlePreview}
                disabled={!file || loading}
                className="btn btn-primary"
              >
                {loading ? 'Procesando...' : 'Continuar →'}
              </button>
            </div>

            <div className="import-help">
              <h4>Formato esperado:</h4>
              <ul>
                <li><strong>Nombre:</strong> Obligatorio, texto</li>
                <li><strong>Precio Profesional:</strong> Número (ej: 1234.56 o 1.234,56)</li>
                <li><strong>Precio Público:</strong> Número</li>
                <li>Si falta un precio, se calcula automáticamente con markup del 30%</li>
                <li>Productos sin ambos precios se importan como "no vendibles"</li>
                <li>Filas completamente vacías se ignoran</li>
              </ul>
            </div>
          </div>
        </div>
      )}

      {step === 'preview' && preview && (
        <div className="import-card">
          <div className="import-step">
            <h3 className="step-title">2. Vista previa de importación</h3>
            
            <div className="preview-summary">
              <div className="summary-item">
                <span className="summary-label">Total de filas:</span>
                <span className="summary-value">{preview.total_rows}</span>
              </div>
              <div className="summary-item">
                <span className="summary-label">Filas válidas:</span>
                <span className="summary-value success">{preview.valid_rows}</span>
              </div>
              <div className="summary-item">
                <span className="summary-label">Productos nuevos:</span>
                <span className="summary-value">{preview.new_products}</span>
              </div>
              <div className="summary-item">
                <span className="summary-label">Actualizaciones:</span>
                <span className="summary-value">{preview.updates}</span>
              </div>
              <div className="summary-item">
                <span className="summary-label">Errores:</span>
                <span className={`summary-value ${preview.errors.length > 0 ? 'error' : ''}`}>
                  {preview.errors.length}
                </span>
              </div>
            </div>

            {preview.calculated_professional_prices > 0 || preview.calculated_public_prices > 0 ? (
              <div className="calculated-prices-info">
                <h4>Precios calculados automáticamente:</h4>
                <ul>
                  {preview.calculated_professional_prices > 0 && (
                    <li>{preview.calculated_professional_prices} precios profesionales calculados (faltaban en Excel)</li>
                  )}
                  {preview.calculated_public_prices > 0 && (
                    <li>{preview.calculated_public_prices} precios públicos calculados (faltaban en Excel)</li>
                  )}
                </ul>
                <p className="help-text">
                  * Calculados con markup del 30% configurable en Ajustes
                </p>
              </div>
            ) : null}

            {preview.products_without_prices > 0 && (
              <div className="warning-box">
                <strong>⚠️ {preview.products_without_prices} productos sin precios</strong>
                <p>Estos productos se importarán como "no vendibles" y no podrán agregarse al carrito hasta que se les asignen precios.</p>
              </div>
            )}

            {preview.errors.length > 0 && (
              <div className="errors-box">
                <h4>Errores detectados:</h4>
                <ul className="errors-list">
                  {preview.errors.slice(0, 10).map((error, idx) => (
                    <li key={idx}>
                      Fila {error.row}: {error.message}
                    </li>
                  ))}
                  {preview.errors.length > 10 && (
                    <li>... y {preview.errors.length - 10} errores más</li>
                  )}
                </ul>
              </div>
            )}

            <div className="import-actions">
              <button onClick={handleReset} className="btn btn-secondary">
                ← Cancelar
              </button>
              <button
                onClick={handleImport}
                disabled={loading}
                className="btn btn-primary"
              >
                {loading ? 'Importando...' : 'Confirmar Importación'}
              </button>
            </div>
          </div>
        </div>
      )}

      {step === 'importing' && (
        <div className="import-card">
          <div className="import-step">
            <div className="loading-spinner">
              <div className="spinner"></div>
              <p>Importando productos...</p>
              <p className="help-text">Esto puede tomar unos segundos</p>
            </div>
          </div>
        </div>
      )}

      {step === 'complete' && result && (
        <div className="import-card">
          <div className="import-step">
            <h3 className="step-title success-title">✓ Importación completada</h3>
            
            <div className="result-summary">
              <div className="result-item">
                <span className="result-label">Productos nuevos:</span>
                <span className="result-value">{result.new_products}</span>
              </div>
              <div className="result-item">
                <span className="result-label">Productos actualizados:</span>
                <span className="result-value">{result.updates}</span>
              </div>
              <div className="result-item">
                <span className="result-label">Total importado:</span>
                <span className="result-value success">{result.valid_rows}</span>
              </div>
            </div>

            <div className="import-actions">
              <button onClick={handleReset} className="btn btn-primary">
                Importar otro archivo
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
