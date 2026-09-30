# Reference
## Astrodynamics
<details><summary><code>client.Astrodynamics.GetOverheadSatellite() -> *sdk.OverheadResponse</code></summary>
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

```go
request := &sdk.GetOverheadSatelliteRequest{
    Lat: 1.1,
    Lon: 1.1,
}
client.Astrodynamics.GetOverheadSatellite(
    context.TODO(),
    request,
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

**lat:** `float64` — Observer latitude in decimal degrees (-90.0 to 90.0)
    
</dd>
</dl>

<dl>
<dd>

**lon:** `float64` — Observer longitude in decimal degrees (-180.0 to 180.0)
    
</dd>
</dl>

<dl>
<dd>

**alt:** `*float64` — Observer altitude above sea level in meters
    
</dd>
</dl>

<dl>
<dd>

**time:** `*time.Time` — UTC timestamp for calculation epoch (defaults to current time if omitted)
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.Astrodynamics.GetNextVisiblePass(ID) -> *sdk.NextVisiblePassResponse</code></summary>
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

```go
request := &sdk.GetNextVisiblePassRequest{
    ID: "id",
    Lat: 1.1,
    Lon: 1.1,
}
client.Astrodynamics.GetNextVisiblePass(
    context.TODO(),
    request,
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

**id:** `string` — Satellite unique UUID identifier
    
</dd>
</dl>

<dl>
<dd>

**lat:** `float64` — Observer latitude in decimal degrees (-90.0 to 90.0)
    
</dd>
</dl>

<dl>
<dd>

**lon:** `float64` — Observer longitude in decimal degrees (-180.0 to 180.0)
    
</dd>
</dl>

<dl>
<dd>

**alt:** `*float64` — Observer altitude above sea level in meters
    
</dd>
</dl>

<dl>
<dd>

**thresholdDeg:** `*float64` — Minimum elevation angle threshold in degrees (default: 5.0 deg)
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

## Pipelines
<details><summary><code>client.Pipelines.TriggerPipelineSync() -> *sdk.PipelineSyncResponse</code></summary>
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

```go
request := &sdk.TriggerPipelineSyncRequest{}
client.Pipelines.TriggerPipelineSync(
    context.TODO(),
    request,
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

**group:** `*string` — CelesTrak satellite group name (e.g. 'stations', 'visual', 'starlink', 'weather', 'active', 'last-30-days')
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

## Satellites
<details><summary><code>client.Satellites.ListSatellites() -> *sdk.PaginatedResponseSatellite</code></summary>
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

```go
request := &sdk.ListSatellitesRequest{}
client.Satellites.ListSatellites(
    context.TODO(),
    request,
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

**limit:** `*int` 
    
</dd>
</dl>

<dl>
<dd>

**cursor:** `*string` 
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.Satellites.CreateSatellite(request) -> *sdk.Satellite</code></summary>
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

```go
request := &sdk.CreateSatelliteDto{
    LineOne: "lineOne",
    LineTwo: "lineTwo",
    Name: "name",
}
client.Satellites.CreateSatellite(
    context.TODO(),
    request,
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

**lineOne:** `string` 
    
</dd>
</dl>

<dl>
<dd>

**lineTwo:** `string` 
    
</dd>
</dl>

<dl>
<dd>

**name:** `string` 
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.Satellites.GetSatellite(ID) -> *sdk.Satellite</code></summary>
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

```go
request := &sdk.GetSatelliteRequest{
    ID: "id",
}
client.Satellites.GetSatellite(
    context.TODO(),
    request,
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

**id:** `string` — Satellite unique UUID identifier
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.Satellites.DeleteSatellite(ID) -> error</code></summary>
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

```go
request := &sdk.DeleteSatelliteRequest{
    ID: "id",
}
client.Satellites.DeleteSatellite(
    context.TODO(),
    request,
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

**id:** `string` — Satellite unique UUID identifier
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

<details><summary><code>client.Satellites.UpdateSatellite(ID, request) -> *sdk.Satellite</code></summary>
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

```go
request := &sdk.UpdateSatelliteDto{
    ID: "id",
}
client.Satellites.UpdateSatellite(
    context.TODO(),
    request,
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

**id:** `string` — Satellite unique UUID identifier
    
</dd>
</dl>

<dl>
<dd>

**lineOne:** `*string` 
    
</dd>
</dl>

<dl>
<dd>

**lineTwo:** `*string` 
    
</dd>
</dl>

<dl>
<dd>

**name:** `*string` 
    
</dd>
</dl>
</dd>
</dl>


</dd>
</dl>
</details>

