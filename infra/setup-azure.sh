#!/usr/bin/env bash
# One-time Azure setup for the UniCloud portal. Run it in Azure Cloud Shell (Bash).
# Creates the resource group, the Container Apps environment, the PostgreSQL database and an
# identity that GitHub Actions deploys with (OIDC, so no password is stored in GitHub).
# The container app itself is created by the pipeline on its first run.
# Safe to re-run: anything that already exists is left as it is.
set -euo pipefail

# ---- edit these ----
GITHUB_REPO="your-github-username/unicloud-portal"  # owner/repo, exactly as in the GitHub URL
LOCATION="germanywestcentral"   # Frankfurt. If Azure says this region isn't allowed for your
                                # subscription, try polandcentral or swedencentral.
PG_LOCATION="$LOCATION"         # change only if PostgreSQL isn't offered in LOCATION
RG="unicloud-rg"                # must match deploy.yml
ENV_NAME="unicloud-env"         # must match deploy.yml
IDENTITY="unicloud-github-deployer"
PG_ADMIN="unicloudadmin"
PG_DB="unicloud"
# --------------------

SUBSCRIPTION_ID=$(az account show --query id -o tsv)
PG_SERVER="unicloud-pg-${SUBSCRIPTION_ID:0:8}"   # unique across Azure, the same on every re-run
echo "Using subscription: $(az account show --query name -o tsv)"

echo "Registering resource providers (a few minutes the first time)..."
az extension add --name containerapp --upgrade --only-show-errors
for ns in Microsoft.App Microsoft.OperationalInsights Microsoft.ManagedIdentity Microsoft.DBforPostgreSQL; do
  az provider register --namespace "$ns" --wait
done

echo "Creating the resource group and Container Apps environment..."
az group show --name "$RG" > /dev/null 2>&1 ||
  az group create --name "$RG" --location "$LOCATION" --output none
az containerapp env show --name "$ENV_NAME" --resource-group "$RG" > /dev/null 2>&1 ||
  az containerapp env create --name "$ENV_NAME" --resource-group "$RG" --location "$LOCATION" --output none

echo "Creating the PostgreSQL server $PG_SERVER (5-10 minutes)..."
DATABASE_URL=""
if az postgres flexible-server show --name "$PG_SERVER" --resource-group "$RG" > /dev/null 2>&1; then
  echo "  Already exists; keeping the password from the first run."
else
  # Letters and digits only, so the password is safe inside a URL. The suffix guarantees
  # the upper-case, lower-case and digit characters Azure requires.
  RAW=$(openssl rand -base64 48 | tr -dc 'A-Za-z0-9')
  PG_PASSWORD="${RAW:0:20}Aa1"
  # B1ms with 32 GB of storage matches the Azure for Students free offer (ADR 0001: PostgreSQL only).
  # --public-access 0.0.0.0 lets services running in Azure (your container app) connect;
  # the password and TLS are still required.
  az postgres flexible-server create     --name "$PG_SERVER" --resource-group "$RG" --location "$PG_LOCATION"     --tier Burstable --sku-name Standard_B1ms --storage-size 32 --version 17     --admin-user "$PG_ADMIN" --admin-password "$PG_PASSWORD"     --public-access 0.0.0.0 --yes --output none
  PG_HOST=$(az postgres flexible-server show --name "$PG_SERVER" --resource-group "$RG"     --query fullyQualifiedDomainName -o tsv)
  # A PostgreSQL URL for the portal, with the user and password included and TLS required
  DATABASE_URL="postgres://${PG_ADMIN}:${PG_PASSWORD}@${PG_HOST}:5432/${PG_DB}?sslmode=require"
fi
az postgres flexible-server db show --server-name "$PG_SERVER" --resource-group "$RG"     --database-name "$PG_DB" > /dev/null 2>&1 ||
  az postgres flexible-server db create --server-name "$PG_SERVER" --resource-group "$RG"     --database-name "$PG_DB" --output none

echo "Creating the identity GitHub Actions deploys with..."
az identity create --name "$IDENTITY" --resource-group "$RG" --location "$LOCATION" --output none
CLIENT_ID=$(az identity show --name "$IDENTITY" --resource-group "$RG" --query clientId -o tsv)
PRINCIPAL_ID=$(az identity show --name "$IDENTITY" --resource-group "$RG" --query principalId -o tsv)

# Trust only GitHub Actions runs on the main branch of your repo
az identity federated-credential create \
  --name github-main \
  --identity-name "$IDENTITY" \
  --resource-group "$RG" \
  --issuer "https://token.actions.githubusercontent.com" \
  --subject "repo:${GITHUB_REPO}:ref:refs/heads/main" \
  --audiences "api://AzureADTokenExchange" \
  --output none

# Let it manage resources in this resource group only. A brand-new identity can take
# a minute to become visible, so retry a few times.
RG_ID=$(az group show --name "$RG" --query id -o tsv)
for attempt in 1 2 3 4 5 6; do
  if OUT=$(az role assignment create --assignee-object-id "$PRINCIPAL_ID" \
        --assignee-principal-type ServicePrincipal --role Contributor \
        --scope "$RG_ID" --output none 2>&1) || grep -qi "already exists" <<< "$OUT"; then
    break
  fi
  if [ "$attempt" -eq 6 ]; then echo "$OUT" >&2; exit 1; fi
  echo "  identity not visible yet, retrying in 20 s..."
  sleep 20
done

echo
echo "Done. Add these as GitHub secrets (repo -> Settings -> Secrets and variables -> Actions):"
echo "  AZURE_CLIENT_ID       = $CLIENT_ID"
echo "  AZURE_TENANT_ID       = $(az account show --query tenantId -o tsv)"
echo "  AZURE_SUBSCRIPTION_ID = $SUBSCRIPTION_ID"
if [ -n "$DATABASE_URL" ]; then
  echo "  DATABASE_URL          = $DATABASE_URL"
  echo
  echo "The database password is shown only this once. Save DATABASE_URL in a password manager now."
fi
