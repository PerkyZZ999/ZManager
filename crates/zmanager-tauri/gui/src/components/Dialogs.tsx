/**
 * Dialog components for file operations
 *
 * Includes:
 * - ConfirmDialog: Generic confirmation with customizable actions
 * - RenameDialog: Inline rename with validation
 * - NewFolderDialog: Create new folder with name input
 */

import clsx from "clsx";
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Dialog Context
// ============================================================================

interface DialogState {
  type: "confirm" | "rename" | "newFolder" | "input";
  props: ConfirmDialogProps | RenameDialogProps | NewFolderDialogProps | InputDialogProps;
}

interface DialogContextValue {
  showConfirm: (props: Omit<ConfirmDialogProps, "onClose">) => Promise<boolean>;
  showRename: (props: Omit<RenameDialogProps, "onClose">) => Promise<string | null>;
  showNewFolder: (props: Omit<NewFolderDialogProps, "onClose">) => Promise<string | null>;
  showInput: (props: Omit<InputDialogProps, "onClose">) => Promise<string | null>;
}

const DialogContext = createContext<DialogContextValue | null>(null);

export function useDialog() {
  const ctx = useContext(DialogContext);
  if (!ctx) {
    throw new Error("useDialog must be used within DialogProvider");
  }
  return ctx;
}

// ============================================================================
// Base Dialog Wrapper
// ============================================================================

interface DialogWrapperProps {
  title: string;
  children: ReactNode;
  onClose: () => void;
  footer?: ReactNode;
  width?: string;
}

function DialogWrapper({ title, children, onClose, footer, width = "w-96" }: DialogWrapperProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [onClose]);

  // Focus trap
  useEffect(() => {
    const focusable = dialogRef.current?.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (focusable && focusable.length > 0) {
      (focusable[0] as HTMLElement).focus();
    }
  }, []);

  return (
    <div className="fixed inset-0 z-200 flex items-center justify-center bg-black/60">
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
        className={clsx(
          width,
          "overflow-hidden rounded-lg border border-zinc-700 bg-zinc-800 shadow-2xl"
        )}
      >
        {/* Header */}
        <div className="flex items-center justify-between border-zinc-700 border-b px-4 py-3">
          <h2 id="dialog-title" className="font-semibold text-lg">
            {title}
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 transition-colors hover:bg-white/10"
            aria-label="Close dialog"
          >
            <SvgIcon name="ic_dismiss" size={16} />
          </button>
        </div>

        {/* Content */}
        <div className="px-4 py-4">{children}</div>

        {/* Footer */}
        {footer && (
          <div className="flex justify-end gap-2 border-zinc-700 border-t px-4 py-3">{footer}</div>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// Button Component
// ============================================================================

interface ButtonProps {
  children: ReactNode;
  onClick?: () => void;
  variant?: "primary" | "secondary" | "danger";
  disabled?: boolean;
  type?: "button" | "submit";
}

function Button({
  children,
  onClick,
  variant = "secondary",
  disabled = false,
  type = "button",
}: ButtonProps) {
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      className={clsx(
        "rounded px-4 py-2 font-medium text-sm transition-colors",
        disabled && "cursor-not-allowed opacity-50",
        variant === "primary" && "bg-primary text-zinc-900 hover:bg-primary/80",
        variant === "secondary" && "bg-zinc-700 text-white hover:bg-zinc-600",
        variant === "danger" && "bg-red-600 text-white hover:bg-red-500"
      )}
    >
      {children}
    </button>
  );
}

// ============================================================================
// Confirm Dialog
// ============================================================================

export interface ConfirmDialogProps {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
  onClose: (confirmed: boolean) => void;
}

function ConfirmDialog({
  title,
  message,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  danger = false,
  onClose,
}: ConfirmDialogProps) {
  return (
    <DialogWrapper
      title={title}
      onClose={() => onClose(false)}
      footer={
        <>
          <Button onClick={() => onClose(false)}>{cancelLabel}</Button>
          <Button variant={danger ? "danger" : "primary"} onClick={() => onClose(true)}>
            {confirmLabel}
          </Button>
        </>
      }
    >
      <p className="text-zinc-300">{message}</p>
    </DialogWrapper>
  );
}

// ============================================================================
// Rename Dialog
// ============================================================================

export interface RenameDialogProps {
  currentName: string;
  isDirectory: boolean;
  onClose: (newName: string | null) => void;
}

function RenameDialog({ currentName, isDirectory, onClose }: RenameDialogProps) {
  const [name, setName] = useState(currentName);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus();
      // Select name without extension for files
      const dotIndex = currentName.lastIndexOf(".");
      if (!isDirectory && dotIndex > 0) {
        inputRef.current.setSelectionRange(0, dotIndex);
      } else {
        inputRef.current.select();
      }
    }
  }, [currentName, isDirectory]);

  const validate = useCallback((value: string): string | null => {
    if (!value.trim()) {
      return "Name cannot be empty";
    }
    if (value.includes("/") || value.includes("\\")) {
      return "Name cannot contain slashes";
    }
    if (/[<>:"|?*]/.test(value)) {
      return 'Name cannot contain special characters: < > : " | ? *';
    }
    return null;
  }, []);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      const validationError = validate(name);
      if (validationError) {
        setError(validationError);
        return;
      }
      if (name === currentName) {
        onClose(null);
        return;
      }
      onClose(name);
    },
    [name, currentName, validate, onClose]
  );

  return (
    <DialogWrapper
      title={`Rename ${isDirectory ? "Folder" : "File"}`}
      onClose={() => onClose(null)}
      footer={
        <>
          <Button onClick={() => onClose(null)}>Cancel</Button>
          <Button
            variant="primary"
            onClick={() => handleSubmit({ preventDefault: () => {} } as React.FormEvent)}
          >
            Rename
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit}>
        <label htmlFor="rename-input" className="mb-2 block text-sm text-zinc-400">
          New name
        </label>
        <input
          id="rename-input"
          ref={inputRef}
          type="text"
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            setError(null);
          }}
          className={clsx(
            "w-full rounded border bg-zinc-900 px-3 py-2 text-white outline-none",
            error ? "border-red-500" : "border-zinc-600 focus:border-primary"
          )}
          spellCheck={false}
        />
        {error && <p className="mt-2 text-red-400 text-sm">{error}</p>}
      </form>
    </DialogWrapper>
  );
}

// ============================================================================
// New Folder Dialog
// ============================================================================

export interface NewFolderDialogProps {
  parentPath?: string;
  onClose: (folderName: string | null) => void;
}

function NewFolderDialog({ parentPath = "", onClose }: NewFolderDialogProps) {
  const [name, setName] = useState("New Folder");
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, []);

  const validate = useCallback((value: string): string | null => {
    if (!value.trim()) {
      return "Folder name cannot be empty";
    }
    if (value.includes("/") || value.includes("\\")) {
      return "Name cannot contain slashes";
    }
    if (/[<>:"|?*]/.test(value)) {
      return 'Name cannot contain special characters: < > : " | ? *';
    }
    return null;
  }, []);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      const validationError = validate(name);
      if (validationError) {
        setError(validationError);
        return;
      }
      onClose(name);
    },
    [name, validate, onClose]
  );

  // Get display path (truncated if too long)
  const displayPath = parentPath.length > 40 ? `...${parentPath.slice(-37)}` : parentPath;

  return (
    <DialogWrapper
      title="New Folder"
      onClose={() => onClose(null)}
      footer={
        <>
          <Button onClick={() => onClose(null)}>Cancel</Button>
          <Button
            variant="primary"
            onClick={() => handleSubmit({ preventDefault: () => {} } as React.FormEvent)}
          >
            Create
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit}>
        <p className="mb-3 text-sm text-zinc-400">
          Create folder in: <span className="text-zinc-300">{displayPath}</span>
        </p>
        <label htmlFor="newfolder-input" className="mb-2 block text-sm text-zinc-400">
          Folder name
        </label>
        <input
          id="newfolder-input"
          ref={inputRef}
          type="text"
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            setError(null);
          }}
          className={clsx(
            "w-full rounded border bg-zinc-900 px-3 py-2 text-white outline-none",
            error ? "border-red-500" : "border-zinc-600 focus:border-primary"
          )}
          spellCheck={false}
        />
        {error && <p className="mt-2 text-red-400 text-sm">{error}</p>}
      </form>
    </DialogWrapper>
  );
}

// ============================================================================
// Input Dialog (Generic)
// ============================================================================

export interface InputDialogProps {
  title: string;
  label: string;
  defaultValue?: string;
  placeholder?: string;
  confirmLabel?: string;
  onClose: (value: string | null) => void;
}

function InputDialog({
  title,
  label,
  defaultValue = "",
  placeholder = "",
  confirmLabel = "OK",
  onClose,
}: InputDialogProps) {
  const [value, setValue] = useState(defaultValue);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, []);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      onClose(value);
    },
    [value, onClose]
  );

  return (
    <DialogWrapper
      title={title}
      onClose={() => onClose(null)}
      footer={
        <>
          <Button onClick={() => onClose(null)}>Cancel</Button>
          <Button
            variant="primary"
            onClick={() => handleSubmit({ preventDefault: () => {} } as React.FormEvent)}
          >
            {confirmLabel}
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit}>
        <label htmlFor="input-dialog-input" className="mb-2 block text-sm text-zinc-400">
          {label}
        </label>
        <input
          id="input-dialog-input"
          ref={inputRef}
          type="text"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder={placeholder}
          className="w-full rounded border border-zinc-600 bg-zinc-900 px-3 py-2 text-white outline-none focus:border-primary"
          spellCheck={false}
        />
      </form>
    </DialogWrapper>
  );
}

// ============================================================================
// Dialog Provider
// ============================================================================

export function DialogProvider({ children }: { children: ReactNode }) {
  const [dialog, setDialog] = useState<DialogState | null>(null);
  const resolverRef = useRef<((value: unknown) => void) | null>(null);

  const showConfirm = useCallback(
    (props: Omit<ConfirmDialogProps, "onClose">): Promise<boolean> => {
      return new Promise((resolve) => {
        resolverRef.current = resolve as (value: unknown) => void;
        setDialog({
          type: "confirm",
          props: {
            ...props,
            onClose: (confirmed: boolean) => {
              setDialog(null);
              resolve(confirmed);
            },
          },
        });
      });
    },
    []
  );

  const showRename = useCallback(
    (props: Omit<RenameDialogProps, "onClose">): Promise<string | null> => {
      return new Promise((resolve) => {
        resolverRef.current = resolve as (value: unknown) => void;
        setDialog({
          type: "rename",
          props: {
            ...props,
            onClose: (newName: string | null) => {
              setDialog(null);
              resolve(newName);
            },
          },
        });
      });
    },
    []
  );

  const showNewFolder = useCallback(
    (props: Omit<NewFolderDialogProps, "onClose">): Promise<string | null> => {
      return new Promise((resolve) => {
        resolverRef.current = resolve as (value: unknown) => void;
        setDialog({
          type: "newFolder",
          props: {
            ...props,
            onClose: (folderName: string | null) => {
              setDialog(null);
              resolve(folderName);
            },
          },
        });
      });
    },
    []
  );

  const showInput = useCallback(
    (props: Omit<InputDialogProps, "onClose">): Promise<string | null> => {
      return new Promise((resolve) => {
        resolverRef.current = resolve as (value: unknown) => void;
        setDialog({
          type: "input",
          props: {
            ...props,
            onClose: (value: string | null) => {
              setDialog(null);
              resolve(value);
            },
          },
        });
      });
    },
    []
  );

  return (
    <DialogContext.Provider value={{ showConfirm, showRename, showNewFolder, showInput }}>
      {children}
      {dialog?.type === "confirm" && <ConfirmDialog {...(dialog.props as ConfirmDialogProps)} />}
      {dialog?.type === "rename" && <RenameDialog {...(dialog.props as RenameDialogProps)} />}
      {dialog?.type === "newFolder" && (
        <NewFolderDialog {...(dialog.props as NewFolderDialogProps)} />
      )}
      {dialog?.type === "input" && <InputDialog {...(dialog.props as InputDialogProps)} />}
    </DialogContext.Provider>
  );
}
