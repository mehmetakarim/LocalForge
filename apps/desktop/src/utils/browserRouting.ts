export interface ProjectRoutingInfo {
  projectUrl: string;
  curlCommand: string;
}

export function computeProjectRouting(
  domain: string,
  port: number = 8080
): ProjectRoutingInfo {
  const cleanDomain = domain.trim();
  return {
    projectUrl: `http://${cleanDomain}:${port}/`,
    curlCommand: `curl -H "Host: ${cleanDomain}" http://127.0.0.1:${port}/`,
  };
}
