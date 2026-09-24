export function shouldRevealMainWindow(args: readonly string[]): boolean {
  return !args.includes("--autostart");
}
