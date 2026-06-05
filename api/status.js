/** CinemaStudio deployment status — no secrets exposed. */
export default function handler(_req, res) {
  const supabaseConfigured = Boolean(
    process.env.SUPABASE_URL && process.env.SUPABASE_ANON_KEY
  );

  res.setHeader("Content-Type", "application/json");
  res.status(200).json({
    service: "cinemastudio-web",
    version: "1.0.0",
    mvp: { phases: "0-12", status: "complete" },
    connections: {
      github: {
        status: "linked",
        repo: "aperezavilez-ai/gaf-cinema-studio",
      },
      vercel: {
        status: "deployed",
        region: process.env.VERCEL_REGION ?? null,
      },
      supabase: {
        status: supabaseConfigured ? "configured" : "pending",
        note: supabaseConfigured
          ? "Env vars present — wire auth/backup when ready"
          : "Add SUPABASE_URL + SUPABASE_ANON_KEY in Vercel project settings",
      },
    },
  });
}
