# GistDB 🌐⚡

[![AGPL-3.0 License](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://opensource.org/licenses/AGPL-3.0)
[![API Version](https://img.shields.io/badge/API%20Version-1.0.0-brightgreen)](https://gist-db.mohammadsadiq4950.workers.dev/docs)

A Rust-powered database that uses GitHub Gists as a storage layer. Simple, reliable, and secure.

### Get Started
```bash
curl https://gist-db.mohammadsadiq4950.workers.dev/
```

## 📌 Features

- **GitHub Gists Integration**: Leverage GitHub's infrastructure for document storage
- **Rust-Powered Backend**: High-performance API server built with Cloudflare Workers
- **Collection Management**: Organize documents into named collections
- **Search Capabilities**: Document search within collections
- **Health Monitoring**: Built-in system health checks and status reporting

## 🚀 Quick Start

### Base URLs
```bash
# Development
http://localhost:8787

# Production
https://gist-db.mohammadsadiq4950.workers.dev
```

## 🔐 Authentication

All protected endpoints require a GitHub Personal Access Token within `gist` scope:

```bash
curl -H "Authorization: Bearer YOUR_GITHUB_TOKEN" \
  https://gist-db.mohammadsadiq4950.workers.dev/api/databases
```

## 📚 API Reference

### 1. **System Information**
#### Get API Information
```http
GET /
```

#### Health Check
```http
GET /health
```
---

### 2. **Database Operations**
#### Create Database
```http
POST /api/databases
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "my_database"
}
```
**Response:**
```json
{
  "status": 201,
  "data": {
    "gist_id": "2b4d4b3e6a04a54d5a9d"
  },
  "message": "Database created successfully",
  "error": null
}
```

#### Delete Database
```http
DELETE /api/databases
Content-Type: application/json
Authorization: Bearer <token>

{
  "gist_id": "2b4d4b3e6a04a54d5a9d"
}
```
**Response:**
```json
{
  "status": 200,
  "data": null,
  "message": "Database deleted successfully",
  "error": null
}
```

---

### 3. **Collection Operations**
#### Create Collection
```http
POST /api/collections
Content-Type: application/json
Authorization: Bearer <token>

{
  "gist_id": "2b4d4b3e6a04a54d5a9d",
  "name": "users"
}
```
**Response:**
```json
{
  "status": 201,
  "data": {
    "collection_name": "users"
  },
  "message": "Collection created successfully",
  "error": null
}
```

#### Delete Collection
```http
DELETE /api/collections
Content-Type: application/json
Authorization: Bearer <token>

{
  "gist_id": "2b4d4b3e6a04a54d5a9d",
  "collection_name": "users"
}
```
**Response:**
```json
{
  "status": 200,
  "data": null,
  "message": "Collection deleted successfully",
  "error": null
}
```

---

### 4. **Document Operations**
#### Create Object
```http
POST /api/objects
Content-Type: application/json
Authorization: Bearer <token>

{
  "gist_id": "2b4d4b3e6a04a54d5a9d",
  "collection_name": "users",
  "data": {
    "name": "Alice",
    "age": 28,
    "email": "alice@example.com"
  }
}
```
**Response:**
```json
{
  "status": 201,
  "data": {
    "object_id": "12345"
  },
  "message": "Object created successfully",
  "error": null
}
```

#### Update Object
```http
PUT /api/objects
Content-Type: application/json
Authorization: Bearer <token>

{
  "gist_id": "2b4d4b3e6a04a54d5a9d",
  "collection_name": "users",
  "object_id": "12345",
  "data": {
    "age": 29
  }
}
```
**Response:**
```json
{
  "status": 200,
  "data": {
    "object_id": "12345"
  },
  "message": "Object updated successfully",
  "error": null
}
```

#### Delete Object
```http
DELETE /api/objects
Content-Type: application/json
Authorization: Bearer <token>

{
  "gist_id": "2b4d4b3e6a04a54d5a9d",
  "collection_name": "users",
  "object_id": "12345"
}
```
**Response:**
```json
{
  "status": 200,
  "data": null,
  "message": "Object deleted successfully",
  "error": null
}
```

---

### 5. **Search Operations**
#### Search Objects
```http
POST /api/search
Content-Type: application/json
Authorization: Bearer <token>

{
  "gist_id": "2b4d4b3e6a04a54d5a9d",
  "collection_name": "users",
  "query": "Alice",
  "field": "name"
}
```
**Response:**
```json
{
  "status": 200,
  "data": [
    {
      "object_id": "12345",
      "data": {
        "name": "Alice",
        "age": 28,
        "email": "alice@example.com"
      }
    }
  ],
  "message": "Search completed",
  "error": null
}
```

---

### 6. **Documentation**
#### Get OpenAPI Specification
```http
GET /docs/openapi.yaml
```
**Response:**  
Returns the OpenAPI specification in YAML format.

#### Interactive API Documentation
```http
GET /docs
```
**Response:**  
Returns the Swagger UI for interactive API exploration.

---

## 🤝 Contributing

Contributions welcome! Please follow guidelines:
1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📜 License

This project is licensed under the [GNU AGPLv3](https://github.com/MdSadiqMd/GistDB/blob/main/LICENSE).
- Any modifications or usage in a networked environment *must be open-sourced*.  
- If you wish to use this project commercially *without open-sourcing modifications*, please contact me for a commercial license.

## 📬 Contact

**Creator and Maintainer:** Md.Sadiq  
- GitHub: [@MdSadiq](https://github.com/MdSadiqMd) 
- X: [@MdSadiq](https://x.com/Md_Sadiq_Md)
