# Reference
## Astrodynamics
<details><summary><code>client.astrodynamics.<a href="src/satellite_api/astrodynamics/client.py">get_overhead_satellite</a>(...) -> OverheadResponse</code></summary>
<dl>
<dd>

#### 📝 Description

<dl>
<dd>

<dl>
<dd>

Computes the satellite closest to overhead (highest elevation) across all tracked satellites using parallel Rayon propagation.
</dd>
</dl>
</dd>
</dl>

#### 🔌 Usage

<dl>
<dd>

<dl>
<dd>

```python
from satellite_api import SatelliteApiApi

client = SatelliteApiApi(
    base_url="https://yourhost.com/path/to/api",
)

client.astrodynamics.get_overhead_satellite(
    lat=1.1,
    lon=1.1,
)

```
</dd>
</dl>
</dd>
</dl>

#### ⚙️ Parameters

<dl>
<dd>

<dl>
<dd>

**lat:** `float` — Observer latitude in decimal degrees (-90.0 to 90.0)
    
</dd>
</dl>

<dl>
<dd>

**lon:** `float` — Observer longitude in decimal degrees (-180.0 to 180.0)
    
</dd>
</dl>

<dl>
<dd>

**alt:** `typing.Optional[float]` — Observer altitude above sea level in meters
    
</dd>
</dl>

<dl>
<dd>

**time:** `typing.Optional[datetime.datetime]` — UTC timestamp for calculation epoch (defaults to current time if omitted)
    
</dd>
</dl>

<dl>
<dd>

**request_options:** `typing.Optional[RequestOptions]` — Request-specific configuration.
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.astrodynamics.<a href="src/satellite_api/astrodynamics/client.py">get_next_visible_pass</a>(...) -> NextVisiblePassResponse</code></summary>
<dl>
<dd>

#### 📝 Description

<dl>
<dd>

<dl>
<dd>

Calculates the next visible pass for a specific satellite above elevation threshold.
</dd>
</dl>
</dd>
</dl>

#### 🔌 Usage

<dl>
<dd>

<dl>
<dd>

```python
from satellite_api import SatelliteApiApi

client = SatelliteApiApi(
    base_url="https://yourhost.com/path/to/api",
)

client.astrodynamics.get_next_visible_pass(
    id="id",
    lat=1.1,
    lon=1.1,
)

```
</dd>
</dl>
</dd>
</dl>

#### ⚙️ Parameters

<dl>
<dd>

<dl>
<dd>

**id:** `str` — Satellite unique UUID identifier
    
</dd>
</dl>

<dl>
<dd>

**lat:** `float` — Observer latitude in decimal degrees (-90.0 to 90.0)
    
</dd>
</dl>

<dl>
<dd>

**lon:** `float` — Observer longitude in decimal degrees (-180.0 to 180.0)
    
</dd>
</dl>

<dl>
<dd>

**alt:** `typing.Optional[float]` — Observer altitude above sea level in meters
    
</dd>
</dl>

<dl>
<dd>

**threshold_deg:** `typing.Optional[float]` — Minimum elevation angle threshold in degrees (default: 5.0 deg)
    
</dd>
</dl>

<dl>
<dd>

**request_options:** `typing.Optional[RequestOptions]` — Request-specific configuration.
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

## Pipelines
<details><summary><code>client.pipelines.<a href="src/satellite_api/pipelines/client.py">trigger_pipeline_sync</a>(...) -> PipelineSyncResponse</code></summary>
<dl>
<dd>

#### 📝 Description

<dl>
<dd>

<dl>
<dd>

Triggers automated CelesTrak discovery pipeline synchronization for a specific satellite group.
</dd>
</dl>
</dd>
</dl>

#### 🔌 Usage

<dl>
<dd>

<dl>
<dd>

```python
from satellite_api import SatelliteApiApi

client = SatelliteApiApi(
    base_url="https://yourhost.com/path/to/api",
)

client.pipelines.trigger_pipeline_sync()

```
</dd>
</dl>
</dd>
</dl>

#### ⚙️ Parameters

<dl>
<dd>

<dl>
<dd>

**group:** `typing.Optional[str]` — CelesTrak satellite group name (e.g. 'stations', 'visual', 'starlink', 'weather', 'active', 'last-30-days')
    
</dd>
</dl>

<dl>
<dd>

**request_options:** `typing.Optional[RequestOptions]` — Request-specific configuration.
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

## Satellites
<details><summary><code>client.satellites.<a href="src/satellite_api/satellites/client.py">list_satellites</a>(...) -> PaginatedResponseSatellite</code></summary>
<dl>
<dd>

#### 📝 Description

<dl>
<dd>

<dl>
<dd>

Retrieves a paginated list of satellites using base64 checkpoint cursors.
</dd>
</dl>
</dd>
</dl>

#### 🔌 Usage

<dl>
<dd>

<dl>
<dd>

```python
from satellite_api import SatelliteApiApi

client = SatelliteApiApi(
    base_url="https://yourhost.com/path/to/api",
)

client.satellites.list_satellites()

```
</dd>
</dl>
</dd>
</dl>

#### ⚙️ Parameters

<dl>
<dd>

<dl>
<dd>

**limit:** `typing.Optional[int]` 
    
</dd>
</dl>

<dl>
<dd>

**cursor:** `typing.Optional[str]` 
    
</dd>
</dl>

<dl>
<dd>

**request_options:** `typing.Optional[RequestOptions]` — Request-specific configuration.
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.satellites.<a href="src/satellite_api/satellites/client.py">create_satellite</a>(...) -> Satellite</code></summary>
<dl>
<dd>

#### 📝 Description

<dl>
<dd>

<dl>
<dd>

Creates a new satellite record from Two-Line Element (TLE) set data.
</dd>
</dl>
</dd>
</dl>

#### 🔌 Usage

<dl>
<dd>

<dl>
<dd>

```python
from satellite_api import SatelliteApiApi

client = SatelliteApiApi(
    base_url="https://yourhost.com/path/to/api",
)

client.satellites.create_satellite(
    line_one="lineOne",
    line_two="lineTwo",
    name="name",
)

```
</dd>
</dl>
</dd>
</dl>

#### ⚙️ Parameters

<dl>
<dd>

<dl>
<dd>

**line_one:** `str` 
    
</dd>
</dl>

<dl>
<dd>

**line_two:** `str` 
    
</dd>
</dl>

<dl>
<dd>

**name:** `str` 
    
</dd>
</dl>

<dl>
<dd>

**request_options:** `typing.Optional[RequestOptions]` — Request-specific configuration.
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.satellites.<a href="src/satellite_api/satellites/client.py">get_satellite</a>(...) -> Satellite</code></summary>
<dl>
<dd>

#### 📝 Description

<dl>
<dd>

<dl>
<dd>

Retrieves details for a specific satellite by its unique UUID.
</dd>
</dl>
</dd>
</dl>

#### 🔌 Usage

<dl>
<dd>

<dl>
<dd>

```python
from satellite_api import SatelliteApiApi

client = SatelliteApiApi(
    base_url="https://yourhost.com/path/to/api",
)

client.satellites.get_satellite(
    id="id",
)

```
</dd>
</dl>
</dd>
</dl>

#### ⚙️ Parameters

<dl>
<dd>

<dl>
<dd>

**id:** `str` — Satellite unique UUID identifier
    
</dd>
</dl>

<dl>
<dd>

**request_options:** `typing.Optional[RequestOptions]` — Request-specific configuration.
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.satellites.<a href="src/satellite_api/satellites/client.py">delete_satellite</a>(...)</code></summary>
<dl>
<dd>

#### 📝 Description

<dl>
<dd>

<dl>
<dd>

Deletes a satellite record by its unique UUID.
</dd>
</dl>
</dd>
</dl>

#### 🔌 Usage

<dl>
<dd>

<dl>
<dd>

```python
from satellite_api import SatelliteApiApi

client = SatelliteApiApi(
    base_url="https://yourhost.com/path/to/api",
)

client.satellites.delete_satellite(
    id="id",
)

```
</dd>
</dl>
</dd>
</dl>

#### ⚙️ Parameters

<dl>
<dd>

<dl>
<dd>

**id:** `str` — Satellite unique UUID identifier
    
</dd>
</dl>

<dl>
<dd>

**request_options:** `typing.Optional[RequestOptions]` — Request-specific configuration.
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.satellites.<a href="src/satellite_api/satellites/client.py">update_satellite</a>(...) -> Satellite</code></summary>
<dl>
<dd>

#### 📝 Description

<dl>
<dd>

<dl>
<dd>

Updates a satellite's name or TLE orbital parameters.
</dd>
</dl>
</dd>
</dl>

#### 🔌 Usage

<dl>
<dd>

<dl>
<dd>

```python
from satellite_api import SatelliteApiApi

client = SatelliteApiApi(
    base_url="https://yourhost.com/path/to/api",
)

client.satellites.update_satellite(
    id="id",
)

```
</dd>
</dl>
</dd>
</dl>

#### ⚙️ Parameters

<dl>
<dd>

<dl>
<dd>

**id:** `str` — Satellite unique UUID identifier
    
</dd>
</dl>

<dl>
<dd>

**line_one:** `typing.Optional[str]` 
    
</dd>
</dl>

<dl>
<dd>

**line_two:** `typing.Optional[str]` 
    
</dd>
</dl>

<dl>
<dd>

**name:** `typing.Optional[str]` 
    
</dd>
</dl>

<dl>
<dd>

**request_options:** `typing.Optional[RequestOptions]` — Request-specific configuration.
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

