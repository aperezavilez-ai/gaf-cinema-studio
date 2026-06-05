const { stubResponse } = require("../../web/lib/connections");

module.exports = function handler(_req, res) {
  res.setHeader("Content-Type", "application/json");
  res.status(501).json(
    stubResponse("cloud-backup", [
      "Create Storage bucket project-backups (private)",
      "Wire presigned upload URL generation",
      "Mobile calls this route after local bundle export",
    ])
  );
};
