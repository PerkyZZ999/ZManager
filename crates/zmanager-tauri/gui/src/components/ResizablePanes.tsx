/**
 * ResizablePanes component
 *
 * Provides a resizable dual-pane layout using react-resizable-panels.
 * Supports keyboard navigation to switch between panes.
 */

import type { ReactNode } from "react";
import { Group, Panel, Separator } from "react-resizable-panels";

interface ResizablePanesProps {
  leftPane: ReactNode;
  rightPane: ReactNode;
  defaultLeftSize?: number;
  minSize?: number;
}

export function ResizablePanes({
  leftPane,
  rightPane,
  defaultLeftSize = 50,
  minSize = 20,
}: ResizablePanesProps) {
  return (
    <Group orientation="horizontal" className="flex-1">
      <Panel defaultSize={defaultLeftSize} minSize={minSize}>
        {leftPane}
      </Panel>

      <Separator className="group relative w-1 bg-zinc-700 transition-colors hover:bg-primary/50 focus:bg-primary/50 focus:outline-none">
        {/* Wider hit area for easier grabbing */}
        <div className="absolute inset-y-0 -right-1 -left-1" />
        {/* Visual indicator on hover */}
        <div className="absolute inset-y-0 left-1/2 w-0.5 -translate-x-1/2 bg-primary opacity-0 transition-opacity group-hover:opacity-100" />
      </Separator>

      <Panel defaultSize={100 - defaultLeftSize} minSize={minSize}>
        {rightPane}
      </Panel>
    </Group>
  );
}
