import type { ReactNode } from "react";

/**
 * The dialog header of the Design canvas artboards: the title, its description under it, and the
 * close button at the right (design/ares-slicer-dialogs/assets/dialogs.css `.dialog-header`).
 */
export function DialogHeader({
  id,
  title,
  description,
  descriptionId,
  close,
  children,
}: {
  id: string;
  title: string;
  description?: ReactNode;
  descriptionId?: string;
  close?: () => void;
  children?: ReactNode;
}) {
  return (
    <header className="dialog-header">
      <div className="dialog-heading">
        {children}
        <h1 id={id}>{title}</h1>
        {description !== undefined && <p id={descriptionId}>{description}</p>}
      </div>
      {close && (
        <button className="button close" type="button" aria-label="Close" onClick={close}>
          {"×"}
        </button>
      )}
    </header>
  );
}
