/**
 * Toast notification system
 *
 * Provides toast notifications with:
 * - Success, error, warning, info variants
 * - Auto-dismiss with configurable timeout
 * - Click to dismiss
 * - Animation support
 */

import clsx from "clsx";
import { createContext, type ReactNode, useCallback, useContext, useEffect, useState } from "react";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Types
// ============================================================================

export type ToastVariant = "success" | "error" | "warning" | "info";

export interface Toast {
  id: string;
  variant: ToastVariant;
  title: string;
  message?: string;
  duration?: number; // ms, default 5000, 0 = no auto-dismiss
  action?: {
    label: string;
    onClick: () => void;
  };
}

interface ToastContextValue {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, "id">) => string;
  removeToast: (id: string) => void;
  success: (title: string, message?: string) => string;
  error: (title: string, message?: string) => string;
  warning: (title: string, message?: string) => string;
  info: (title: string, message?: string) => string;
}

// ============================================================================
// Context
// ============================================================================

const ToastContext = createContext<ToastContextValue | null>(null);

// ============================================================================
// Toast Item Component
// ============================================================================

interface ToastItemProps {
  toast: Toast;
  onDismiss: (id: string) => void;
}

function getVariantStyles(variant: ToastVariant) {
  switch (variant) {
    case "success":
      return {
        bg: "bg-green-900/90",
        border: "border-green-500/30",
        icon: "/icons/ui/ic_check_circle.svg",
        iconClass: "text-green-400",
      };
    case "error":
      return {
        bg: "bg-red-900/90",
        border: "border-red-500/30",
        icon: "/icons/ui/ic_error.svg",
        iconClass: "text-red-400",
      };
    case "warning":
      return {
        bg: "bg-yellow-900/90",
        border: "border-yellow-500/30",
        icon: "/icons/ui/ic_warning.svg",
        iconClass: "text-yellow-400",
      };
    case "info":
      return {
        bg: "bg-blue-900/90",
        border: "border-blue-500/30",
        icon: "/icons/ui/ic_info.svg",
        iconClass: "text-blue-400",
      };
  }
}

function ToastItem({ toast, onDismiss }: ToastItemProps) {
  const styles = getVariantStyles(toast.variant);

  useEffect(() => {
    if (toast.duration !== 0) {
      const timeout = setTimeout(() => {
        onDismiss(toast.id);
      }, toast.duration ?? 5000);
      return () => clearTimeout(timeout);
    }
  }, [toast.id, toast.duration, onDismiss]);

  return (
    <div
      className={clsx(
        "flex items-start gap-3 rounded-lg border p-4 shadow-lg backdrop-blur-sm",
        "animate-slide-in-right",
        styles.bg,
        styles.border
      )}
      role="alert"
    >
      <SvgIcon src={styles.icon} size={20} alt={toast.variant} className={styles.iconClass} />
      <div className="min-w-0 flex-1">
        <p className="font-medium text-sm">{toast.title}</p>
        {toast.message && <p className="mt-1 text-sm text-white/70">{toast.message}</p>}
        {toast.action && (
          <button
            type="button"
            onClick={() => {
              toast.action?.onClick();
              onDismiss(toast.id);
            }}
            className="mt-2 font-medium text-sm underline underline-offset-2 hover:no-underline"
          >
            {toast.action.label}
          </button>
        )}
      </div>
      <button
        type="button"
        onClick={() => onDismiss(toast.id)}
        className="rounded p-1 transition-colors hover:bg-white/10"
        aria-label="Dismiss"
      >
        <SvgIcon src="/icons/ui/ic_close.svg" size={16} alt="Close" />
      </button>
    </div>
  );
}

// ============================================================================
// Toast Container Component
// ============================================================================

function ToastContainer({
  toasts,
  onDismiss,
}: {
  toasts: Toast[];
  onDismiss: (id: string) => void;
}) {
  if (toasts.length === 0) return null;

  return (
    <div className="pointer-events-none fixed top-4 right-4 z-[100] flex w-80 flex-col gap-2">
      {toasts.map((toast) => (
        <div key={toast.id} className="pointer-events-auto">
          <ToastItem toast={toast} onDismiss={onDismiss} />
        </div>
      ))}
    </div>
  );
}

// ============================================================================
// Toast Provider Component
// ============================================================================

interface ToastProviderProps {
  children: ReactNode;
}

export function ToastProvider({ children }: ToastProviderProps) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const generateId = useCallback(() => {
    return `toast-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
  }, []);

  const addToast = useCallback(
    (toast: Omit<Toast, "id">) => {
      const id = generateId();
      setToasts((prev) => [...prev, { ...toast, id }]);
      return id;
    },
    [generateId]
  );

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const success = useCallback(
    (title: string, message?: string) => addToast({ variant: "success", title, message }),
    [addToast]
  );

  const error = useCallback(
    (title: string, message?: string) => addToast({ variant: "error", title, message }),
    [addToast]
  );

  const warning = useCallback(
    (title: string, message?: string) => addToast({ variant: "warning", title, message }),
    [addToast]
  );

  const info = useCallback(
    (title: string, message?: string) => addToast({ variant: "info", title, message }),
    [addToast]
  );

  const contextValue: ToastContextValue = {
    toasts,
    addToast,
    removeToast,
    success,
    error,
    warning,
    info,
  };

  return (
    <ToastContext.Provider value={contextValue}>
      {children}
      <ToastContainer toasts={toasts} onDismiss={removeToast} />
    </ToastContext.Provider>
  );
}

// ============================================================================
// Hook
// ============================================================================

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return context;
}
