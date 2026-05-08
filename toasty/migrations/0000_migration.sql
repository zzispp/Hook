CREATE TABLE "models" (
    "id" VARCHAR(36) NOT NULL,
    "provider_id" VARCHAR(36) NOT NULL,
    "global_model_id" VARCHAR(36) NOT NULL,
    "provider_model_name" VARCHAR(200) NOT NULL,
    "provider_model_mappings" TEXT,
    "price_per_request" NUMERIC(20, 8),
    "tiered_pricing" TEXT,
    "supports_vision" BOOLEAN,
    "supports_function_calling" BOOLEAN,
    "supports_streaming" BOOLEAN,
    "supports_extended_thinking" BOOLEAN,
    "supports_image_generation" BOOLEAN,
    "is_active" BOOLEAN NOT NULL,
    "is_available" BOOLEAN NOT NULL,
    "config" TEXT,
    "created_at" TIMESTAMPTZ(6) NOT NULL,
    "updated_at" TIMESTAMPTZ(6) NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_models_by_id" ON "models" ("id");
CREATE INDEX "index_models_by_provider_id" ON "models" ("provider_id");
CREATE INDEX "index_models_by_global_model_id" ON "models" ("global_model_id");
CREATE TABLE "global_models" (
    "id" VARCHAR(36) NOT NULL,
    "name" VARCHAR(100) NOT NULL,
    "display_name" VARCHAR(100) NOT NULL,
    "default_price_per_request" NUMERIC(20, 8),
    "default_tiered_pricing" TEXT NOT NULL,
    "supported_capabilities" TEXT,
    "config" TEXT,
    "is_active" BOOLEAN NOT NULL,
    "usage_count" BIGINT NOT NULL,
    "created_at" TIMESTAMPTZ(6) NOT NULL,
    "updated_at" TIMESTAMPTZ(6) NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_global_models_by_id" ON "global_models" ("id");
CREATE UNIQUE INDEX "index_global_models_by_name" ON "global_models" ("name");
CREATE INDEX "index_global_models_by_usage_count" ON "global_models" ("usage_count");
