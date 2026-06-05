/** Shared deployment status builder — no secrets. */

const GITHUB_REPO = "aperezavilez-ai/gaf-cinema-studio";
const MVP_VERSION = "1.0.0";

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
    service: "cinemastudio-web",
    version: MVP_VERSION,
    mvp: { phases: "0-12", status: "complete" },
    connections: {
      github: { status: "linked", repo: GITHUB_REPO },
      vercel: {
        status: "deployed",
        region: process.env.VERCEL_REGION ?? null,
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

module.exports = { GITHUB_REPO, MVP_VERSION, buildDeploymentStatus, stubResponse };
