export type PortStatus = "valid" | "unassigned" | "invalid_range";

export interface RoutingOptions {
  legacyMode?: boolean;
}

export interface ProjectRoutingInfo {
  projectUrl: string | null;
  preferredUrl: string | null;
  loopbackUrl: string | null;
  localhostUrl: string | null;
  curlCommand: string;
  canOpenBrowser: boolean;
  portStatus: PortStatus;
}

export function computeProjectRouting(
  domain: string,
  port: number = 8080,
  projectName?: string,
  loopbackPort?: number | null,
  options?: RoutingOptions
): ProjectRoutingInfo {
  const cleanDomain = domain.trim();
  const slug = (projectName || cleanDomain.replace(/\.test$/, ""))
    .toLowerCase()
    .replace(/[^a-z0-9]/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");

  const isLegacy = options?.legacyMode === true;
  const isPortProvided =
    loopbackPort !== undefined && loopbackPort !== null && loopbackPort > 0;

  if (!isPortProvided) {
    if (isLegacy) {
      const preferredUrl = `http://${slug}.localforge/`;
      const localhostUrl = `http://${slug}.localforge.localhost:${port}/`;
      const loopbackUrl = `http://127.0.0.1:${port}/`;
      const projectUrl = `http://${cleanDomain}:${port}/`;
      const curlCommand = projectName
        ? `curl -s -H "Host: ${slug}.localforge" http://127.0.0.1:${port}/`
        : `curl -H "Host: ${cleanDomain}" http://127.0.0.1:${port}/`;

      return {
        projectUrl,
        preferredUrl,
        loopbackUrl,
        localhostUrl,
        curlCommand,
        canOpenBrowser: true,
        portStatus: "valid",
      };
    }

    return {
      projectUrl: null,
      preferredUrl: null,
      loopbackUrl: null,
      localhostUrl: null,
      curlCommand: projectName
        ? `curl -s -H "Host: ${slug}.localforge" (port unassigned)`
        : `curl -H "Host: ${cleanDomain}" (port unassigned)`,
      canOpenBrowser: false,
      portStatus: "unassigned",
    };
  }

  // Port is provided: validate range strictly 18081..=18999
  if (loopbackPort < 18081 || loopbackPort > 18999) {
    return {
      projectUrl: null,
      preferredUrl: null,
      loopbackUrl: null,
      localhostUrl: null,
      curlCommand: `# Invalid port ${loopbackPort} (must be between 18081 and 18999)`,
      canOpenBrowser: false,
      portStatus: "invalid_range",
    };
  }

  const effectivePort = loopbackPort;
  const preferredUrl = `http://${slug}.localforge/`;
  const localhostUrl = `http://${slug}.localforge.localhost:${effectivePort}/`;
  const loopbackUrl = `http://127.0.0.1:${effectivePort}/`;
  const projectUrl = `http://${cleanDomain}:${effectivePort}/`;

  const curlCommand = projectName
    ? `curl -s -H "Host: ${slug}.localforge" http://127.0.0.1:${effectivePort}/`
    : `curl -H "Host: ${cleanDomain}" http://127.0.0.1:${effectivePort}/`;

  return {
    projectUrl,
    preferredUrl,
    loopbackUrl,
    localhostUrl,
    curlCommand,
    canOpenBrowser: true,
    portStatus: "valid",
  };
}
