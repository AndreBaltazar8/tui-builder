import { beforeEach, describe, expect, it } from "vitest";
import { createSampleProject, palette } from "./sample";
import { useBuilder } from "./store";
import { findNode } from "./types";

describe("builder store", () => {
  beforeEach(() => {
    const project = createSampleProject();
    useBuilder.setState({ project, selectedFrameId: project.frames[0].id, selectedNodeId: project.frames[0].root.id, past: [], future: [] });
  });

  it("adds a widget to the selected layout and supports undo", () => {
    const before = useBuilder.getState().project.frames[0].root;
    expect(before.type).toBe("layout");
    const paragraph = palette.find((item) => item.name === "Paragraph")!;
    useBuilder.getState().addWidget(paragraph.config(), paragraph.name);
    const state = useBuilder.getState();
    const addedId = state.selectedNodeId!;
    expect(findNode(state.project.frames[0].root, addedId)).toMatchObject({ type: "widget", name: "Paragraph" });
    state.undo();
    expect(findNode(useBuilder.getState().project.frames[0].root, addedId)).toBeUndefined();
  });

  it("duplicates nodes with fresh ids", () => {
    const frame = useBuilder.getState().project.frames[0];
    const firstChild = frame.root.type === "layout" ? frame.root.children[0].node : frame.root;
    useBuilder.getState().duplicateNode(firstChild.id);
    expect(useBuilder.getState().selectedNodeId).not.toBe(firstChild.id);
  });

  it("creates an editable overlay and removes it through its root", () => {
    useBuilder.getState().addOverlay();
    const state = useBuilder.getState();
    const frame = state.project.frames[0];
    expect(frame.overlays).toHaveLength(1);
    expect(state.selectedNodeId).toBe(frame.overlays[0].root.id);
    state.removeNode(frame.overlays[0].root.id);
    expect(useBuilder.getState().project.frames[0].overlays).toHaveLength(0);
  });
});
