import { atom } from "jotai";
import type { InputSourceDto } from "../types";

export type InputSourceSourcesState = {
  sources: InputSourceDto[];
  locale: string | null;
  status: "idle" | "loading" | "ready" | "error";
  fetchedAt: number | null;
};

export const initialInputSourceSourcesState: InputSourceSourcesState = {
  sources: [],
  locale: null,
  status: "idle",
  fetchedAt: null,
};

export const inputSourceSourcesAtom = atom<InputSourceSourcesState>(initialInputSourceSourcesState);
