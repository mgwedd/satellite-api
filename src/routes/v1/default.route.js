const express = require('express');

const router = express.Router();

// This is a temporary catch all route
router.route(['/', '/api', '/api/v1', '/health', '/healthcheck', '/monitoring']).get((req, res) =>
  res.render('index', {
    title: 'The Satellite API',
    message: 'Use the /api/v1/satellite (GET/POST), /api/v1/satellite/overhead (GET), /api/v1/satellite/:id/next-visible, and /api/v1/satellite/:id (GET/PATCH/DELETE) resource paths to CRUD satellites and get satellite visibility information based on TLE data',
  })
);

module.exports = router;
