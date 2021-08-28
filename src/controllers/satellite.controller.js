const httpStatus = require('http-status');
const mongoose = require('mongoose');
const satelliteJS = require('satellite.js');

const ApiError = require('../utils/ApiError');
const catchAsync = require('../utils/catchAsync');
const { satelliteService } = require('../services');

const createSatellitesFromTLE = catchAsync(async (req, res) => {
  res.status(httpStatus.NOT_IMPLEMENTED);
  // const satellites = await satelliteService.createSatellitesFromTLE(req.files.tle);
  // res.status(httpStatus.CREATED).send(satellites);
});

const createSatellite = catchAsync(async (req, res) => {
  const satellite = await satelliteService.createSatellite(req.body);
  res.status(httpStatus.CREATED).send(satellite);
});

const listSatellites = catchAsync(async (req, res) => {
  // TODO use redis cache before looking up in mongodb
  // TODO Add filtering and pagination options in a production-ready version of this server so the API can scale
  const satellitesList = await satelliteService.listSatellites();
  res.send(satellitesList);
});

const getSatellite = catchAsync(async (req, res) => {
  // TODO use redis cache before looking up in mongodb
  const satellite = await satelliteService.getSatelliteById(mongoose.Types.ObjectId(req.params.id));
  if (!satellite) {
    throw new ApiError(httpStatus.NOT_FOUND, 'Satellite not found');
  }
  res.send(satellite);
});

const updateSatellite = catchAsync(async (req, res) => {
  const updatedSatellite = await satelliteService.updateSatelliteById(mongoose.Types.ObjectId(req.params.id), req.body);
  res.send(updatedSatellite);
});

const deleteSatellite = catchAsync(async (req, res) => {
  await satelliteService.deleteSatelliteById(mongoose.Types.ObjectId(req.params.id));
  res.status(httpStatus.NO_CONTENT).send();
});

const getNextVisible = catchAsync(async (req, res) => {
  res.send(httpStatus.NOT_IMPLEMENTED);
  const { id } = req.params;
  const { groundPosition } = req.query.params;
  // TODO use redis cache before looking up in mongodb, looking up request params

  const satellite = await satelliteService.getSatelliteById(mongoose.Types.ObjectId(req.params.id));
  if (!satellite) {
    throw new ApiError(httpStatus.NOT_FOUND, 'Satellite not found');
  }

  // PSEUDOCODE — ran out of time //

  // TODO determine whether the satellite will ever pass over the groundPosition by seeing whether the given groundPosition
  // is ever a point contained within the polygon on the ground from which the satellite is visible at any given point in one full orbit of earth.
  // I'm not sure which of the satellite.js methods is appropriate for this yet.
  // If it never flies over the observer, then return a 404

  // TODO set a threshold for visibility, maybe 5 degrees elevation to account for mountains / trees etc

  // TODO take the total duration of a full earth orbit for the satellite

  // TODO while we have not yet found an elevation for the satellite that meets the threshold...
  // TODO starting at the current time, calculate the elevation of the satellite as below in the getOverhead controller method
  // exit condition - If the elevation meets the threshold, break out of the while loop and return this point in time as the next pass
  // continue condition - if this current elevation is below the threshold still, then continue to the next minute in the orbit duration
});

const getOverhead = catchAsync(async (req, res) => {
  const { groundPosition, time } = req.query.params; // time = timeSinceTleEpochMinutes
  // TODO use redis cache before looking up in mongodb, looking up request params

  const satellitesList = await satelliteService.listSatellites();

  const observerGd = {
    longitude: satelliteJS.degreesToRadians(groundPosition.lon),
    latitude: satelliteJS.degreesToRadians(groundPosition.lat),
    height: 0.37,
  };

  const gmst = satelliteJS.gstime(new Date(time));

  const elevations = [];
  satellitesList.forEach((satellite) => {
    const { position: positionEci } = satelliteJS.sgp4(satellite.satrec, time);
    const positionEcf = satelliteJS.eciToEcf(positionEci, gmst);
    const { elevation } = satelliteJS.ecfToLookAngles(observerGd, positionEcf);
    elevations.push({ satellite, elevation });
  });

  const overheadElevation = 90;

  const closestToOverhead = elevations.reduce((a, b) => {
    return Math.abs(b.elevation - overheadElevation) < Math.abs(a.elevation - overheadElevation) ? b : a;
  });

  res.send(closestToOverhead);
});

module.exports = {
  createSatellite,
  createSatellitesFromTLE,
  listSatellites,
  getSatellite,
  updateSatellite,
  deleteSatellite,
  getNextVisible,
  getOverhead,
};
