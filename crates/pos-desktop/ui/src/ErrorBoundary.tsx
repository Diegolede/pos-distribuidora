import { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
      errorInfo: null,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Error capturado por ErrorBoundary:', error, errorInfo);
    this.setState({
      error,
      errorInfo,
    });
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100vh',
          backgroundColor: '#1a1a1a',
          color: '#fff',
          padding: '2rem',
          fontFamily: 'system-ui, -apple-system, sans-serif',
        }}>
          <div style={{
            maxWidth: '800px',
            width: '100%',
            backgroundColor: '#2a2a2a',
            borderRadius: '8px',
            padding: '2rem',
            border: '1px solid #ef4444',
          }}>
            <h1 style={{ 
              color: '#ef4444', 
              marginBottom: '1rem',
              fontSize: '1.5rem',
              fontWeight: 'bold',
            }}>
              ⚠️ Error en la aplicación
            </h1>
            <p style={{ marginBottom: '1rem', fontSize: '1rem' }}>
              La aplicación ha encontrado un error inesperado.
            </p>
            
            {this.state.error && (
              <div style={{
                backgroundColor: '#1a1a1a',
                padding: '1rem',
                borderRadius: '4px',
                marginBottom: '1rem',
                fontFamily: 'monospace',
                fontSize: '0.875rem',
                overflowX: 'auto',
              }}>
                <strong>Error:</strong> {this.state.error.toString()}
              </div>
            )}

            {this.state.errorInfo && (
              <details style={{ marginBottom: '1rem' }}>
                <summary style={{ 
                  cursor: 'pointer', 
                  marginBottom: '0.5rem',
                  color: '#60a5fa',
                }}>
                  Ver stack trace
                </summary>
                <pre style={{
                  backgroundColor: '#1a1a1a',
                  padding: '1rem',
                  borderRadius: '4px',
                  fontSize: '0.75rem',
                  overflowX: 'auto',
                  whiteSpace: 'pre-wrap',
                }}>
                  {this.state.errorInfo.componentStack}
                </pre>
              </details>
            )}

            <div style={{ marginTop: '1.5rem' }}>
              <button
                onClick={() => window.location.reload()}
                style={{
                  backgroundColor: '#3b82f6',
                  color: 'white',
                  padding: '0.75rem 1.5rem',
                  borderRadius: '4px',
                  border: 'none',
                  fontSize: '1rem',
                  cursor: 'pointer',
                  fontWeight: 'bold',
                }}
                onMouseOver={(e) => e.currentTarget.style.backgroundColor = '#2563eb'}
                onMouseOut={(e) => e.currentTarget.style.backgroundColor = '#3b82f6'}
              >
                Recargar aplicación
              </button>
            </div>

            <div style={{
              marginTop: '1.5rem',
              padding: '1rem',
              backgroundColor: '#1a1a1a',
              borderRadius: '4px',
              fontSize: '0.875rem',
            }}>
              <p style={{ marginBottom: '0.5rem' }}>
                <strong>Posibles soluciones:</strong>
              </p>
              <ul style={{ paddingLeft: '1.5rem', margin: 0 }}>
                <li>Verificar que la base de datos esté accesible</li>
                <li>Reiniciar la aplicación</li>
                <li>Si el problema persiste, verificar los logs de la aplicación</li>
              </ul>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
