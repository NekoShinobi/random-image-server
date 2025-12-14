# Random Image Server

A simple web server built with Actix Web that serves random images from your desired directory.

## Features

- 🎲 Serves random images from your desired directory
- 🖼️ Supports common image formats: JPG, JPEG, PNG, GIF, WebP, BMP
- ⚡ Fast and lightweight using Actix Web framework

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

The server will start on `http://0.0.0.0:8080`

## API Endpoints

### GET /
Returns a random image from the `$IMAGE_DIR` directory.

**Example:**
```bash
curl http://localhost:8080/ --output image.jpg
```

### GET /health
Health check endpoint.

**Example:**
```bash
curl http://localhost:8080/health
```

## Docker

```bash
# Container currently just runs with 1000:1000, so make sure it has permissions to read it if you're running standard docker.
export IMAGE_DIR=${PWD}/images
docker compose up
```

## License

See [LICENSE](./LICENSE) file for details.
