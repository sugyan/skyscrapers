import { useEffect, useRef } from "react";

interface ConfirmDialogProps {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  title,
  message,
  confirmLabel = "OK",
  cancelLabel = "Cancel",
  destructive = false,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  // The cleanup-driven dialog.close() fires a close event we must ignore;
  // otherwise React StrictMode's mount → cleanup → remount cycle treats the
  // synthetic teardown as a user cancel and snaps the dialog shut.
  const closingByCleanup = useRef(false);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    dialog.showModal();
    return () => {
      if (!dialog.open) return;
      closingByCleanup.current = true;
      dialog.close();
    };
  }, []);

  const handleClose = () => {
    if (closingByCleanup.current) {
      closingByCleanup.current = false;
      return;
    }
    onCancel();
  };

  const cancelBtn =
    "px-4 py-2 text-sm font-medium border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700";
  const confirmBtn = destructive
    ? "px-4 py-2 text-sm font-medium border border-red-600 dark:border-red-500 rounded-md bg-red-600 dark:bg-red-600 text-white cursor-pointer hover:bg-red-700 dark:hover:bg-red-700"
    : "px-4 py-2 text-sm font-medium border border-blue-500 dark:border-blue-400 rounded-md bg-blue-500 dark:bg-blue-600 text-white cursor-pointer hover:bg-blue-600 dark:hover:bg-blue-700";

  return (
    <dialog
      ref={dialogRef}
      onClose={handleClose}
      onClick={(e) => {
        if (e.target === e.currentTarget) onCancel();
      }}
      className="backdrop:bg-black/50 bg-transparent p-4 max-w-sm w-full m-auto"
    >
      <div className="bg-white dark:bg-slate-800 rounded-lg p-6 text-gray-900 dark:text-gray-100">
        <h2 className="text-lg font-semibold mb-2">{title}</h2>
        <p className="text-sm leading-relaxed mb-5">{message}</p>
        <div className="flex justify-end gap-2">
          <button
            onClick={onCancel}
            className={cancelBtn}
            autoFocus={destructive}
          >
            {cancelLabel}
          </button>
          <button
            onClick={onConfirm}
            className={confirmBtn}
            autoFocus={!destructive}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </dialog>
  );
}
