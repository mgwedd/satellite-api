const express = require('express');

const router = express.Router();

// This is a temporary catch all route
router.route(['/', 'health', '/healthcheck', '/monitoring']).get((req, res) =>
  res.render('index', {
    title: 'The Satellite API',
    message:
      'Use the /satellite/overhead (GET) and /satellite/:id (GET/PATCH/DELETE) resource paths to CRUD satellites and get satellite visibility information based on TLE data',
  })
);

module.exports = router;
