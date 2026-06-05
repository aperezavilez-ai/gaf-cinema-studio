const { stubResponse } = require("../../web/lib/connections");

module.exports = function handler(_req, res) {
  res.setHeader("Content-Type", "application/json");
  res.status(501).json(
    stubResponse("stripe-webhook", [
      "Create Stripe product + price for CinemaStudio Pro",
      "Add STRIPE_WEBHOOK_SECRET to Vercel",
      "Verify signature and update subscription in Supabase",
    ])
  );
};
