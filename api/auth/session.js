const { stubResponse } = require("../../web/lib/connections");

module.exports = function handler(_req, res) {
  res.setHeader("Content-Type", "application/json");
  res.status(501).json(
    stubResponse("supabase-auth", [
      "Create Supabase project",
      "Run supabase/migrations/001_initial.sql",
      "Add SUPABASE_URL + keys to Vercel",
      "Replace stub with @supabase/supabase-js in this route",
    ])
  );
};
