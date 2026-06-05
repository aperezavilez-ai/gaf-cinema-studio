const { buildDeploymentStatus } = require("../web/lib/connections");

/** CinemaStudio deployment status — no secrets exposed. */
module.exports = function handler(_req, res) {
  res.setHeader("Content-Type", "application/json");
  res.status(200).json(buildDeploymentStatus());
};
