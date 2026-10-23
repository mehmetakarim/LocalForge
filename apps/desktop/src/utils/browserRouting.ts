export interface ProjectRoutingInfo {
  projectUrl: string;
  preferredUrl: string;
  loopbackUrl: string;
  localhostUrl: string;
  curlCommand: string;
}

export function computeProjectRouting(
  domain: string,
  port: number = 8080,
  projectName?: string,
  loopbackPort?: number
): ProjectRoutingInfo {
  const cleanDomain = domain.trim();
  const slug = (projectName || cleanDomain.replace(/\.test$/, ""))
    .toLowerCase()
    .replace(/[^a-z0-9]/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");

  const effectivePort = loopbackPort || port;
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
  };
}
