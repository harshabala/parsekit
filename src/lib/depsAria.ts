/** Accessible status line for optional-converter rows (Settings preflight). */
export function depRowAriaLabel(
  name: string,
  installed: boolean,
  statusInstalled: string,
  statusMissing: string
): string {
  return `${name}: ${installed ? statusInstalled : statusMissing}`;
}