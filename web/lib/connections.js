/** Shared deployment status builder — no secrets. */

const GITHUB_REPO = "aperezavilez-ai/gaf-cinema-studio";
const MVP_VERSION = "1.0.0";
const CUSTOM_DOMAIN =
  process.env.CINEMASTUDIO_PUBLIC_URL || "https://gafcinemastudio.com";
const VERCEL_DOMAIN = "https://gaf-cinema-studio.vercel.app";

function supabaseState() {
  const configured = Boolean(
    process.env.SUPABASE_URL && process.env.SUPABASE_ANON_KEY
  );
  return {
    status: configured ? "configured" : "pending",
    note: configured
      ? "Env vars present — wire auth/backup when ready"
      : "Add SUPABASE_URL + SUPABASE_ANON_KEY in Vercel project settings",
  };
}

function buildDeploymentStatus() {
  return {
    service: "gaf-cinema-studio",
    version: MVP_VERSION,
    mvp: { phases: "0-12", status: "complete" },
    domains: {
      primary: CUSTOM_DOMAIN,
      vercel: VERCEL_DOMAIN,
    },
    connections: {
      github: { status: "linked", repo: GITHUB_REPO },
      vercel: {
        status: "deployed",
        region: process.env.VERCEL_REGION ?? null,
      },
      customDomain: {
        host: new URL(CUSTOM_DOMAIN).hostname,
        status: "configured",
        note: "Set CINEMASTUDIO_PUBLIC_URL in Vercel if using another domain",
      },
      supabase: supabaseState(),
    },
  };
}

function stubResponse(name, connectSteps) {
  return {
    ok: false,
    wired: false,
    service: name,
    message: `${name} not connected — scaffold only`,
    connectSteps,
  };
}

module.exports = {
  GITHUB_REPO,
  MVP_VERSION,
  CUSTOM_DOMAIN,
  VERCEL_DOMAIN,
  buildDeploymentStatus,
  stubResponse,
};
