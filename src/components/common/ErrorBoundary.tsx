import { Component, ErrorInfo, ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * Error Boundary component to catch and display React errors gracefully
 * Prevents the entire app from crashing when a component throws an error
 */
class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    // Update state so the next render will show the fallback UI
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error details for debugging
    console.error("ErrorBoundary caught an error:", error, errorInfo);
    this.setState({ errorInfo });
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="w-full h-full bg-discord-darkest flex items-center justify-center p-6">
          <div className="bg-discord-dark rounded-lg border border-discord-danger p-8 max-w-2xl">
            <div className="flex items-center gap-3 mb-4">
              <span className="text-5xl">⚠️</span>
              <div>
                <h1 className="text-2xl font-bold text-discord-danger">
                  Something went wrong
                </h1>
                <p className="text-discord-text-muted text-sm mt-1">
                  The application encountered an unexpected error
                </p>
              </div>
            </div>

            {this.state.error && (
              <div className="bg-discord-darker rounded p-4 mb-4">
                <p className="text-discord-text font-mono text-sm mb-2">
                  <strong>Error:</strong> {this.state.error.toString()}
                </p>
                {this.state.errorInfo && (
                  <details className="text-discord-text-muted font-mono text-xs">
                    <summary className="cursor-pointer hover:text-discord-text">
                      Stack trace
                    </summary>
                    <pre className="mt-2 overflow-auto max-h-48">
                      {this.state.errorInfo.componentStack}
                    </pre>
                  </details>
                )}
              </div>
            )}

            <div className="flex gap-3">
              <button
                onClick={this.handleReset}
                className="px-4 py-2 bg-discord-primary hover:bg-discord-primary-hover
                         rounded-lg text-white font-medium transition-colors"
              >
                Try Again
              </button>
              <button
                onClick={() => window.location.reload()}
                className="px-4 py-2 bg-discord-dark hover:bg-discord-darker
                         rounded-lg text-discord-text font-medium transition-colors
                         border border-discord-darker"
              >
                Reload Application
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
