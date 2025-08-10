#!/bin/bash

# FinBERT API Service Management Script
# This script provides easy management of the FinBERT API systemd service

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVICE_NAME="finbert-api"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "FinBERT API Service Management"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start     - Start the FinBERT API service"
    echo "  stop      - Stop the FinBERT API service"
    echo "  restart   - Restart the FinBERT API service"
    echo "  status    - Show service status"
    echo "  logs      - Show service logs (follow mode)"
    echo "  logs-all  - Show all service logs"
    echo "  enable    - Enable service to start on boot"
    echo "  disable   - Disable service from starting on boot"
    echo "  reload    - Reload service configuration"
    echo "  health    - Check API health endpoint"
    echo "  metrics   - Show API metrics"
    echo "  help      - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start"
    echo "  $0 status"
    echo "  $0 logs"
    echo "  $0 health"
}

# Function to check if service exists
check_service_exists() {
    if ! systemctl list-unit-files | grep -q "^${SERVICE_NAME}.service"; then
        print_error "Service ${SERVICE_NAME} is not installed!"
        echo "Please run the installation script first:"
        echo "sudo ./install-service.sh"
        exit 1
    fi
}

# Function to check if running as root (for certain commands)
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This command must be run as root (use sudo)"
        exit 1
    fi
}

# Function to check API health
check_health() {
    print_status "Checking API health..."
    if curl -s -f http://localhost:3000/health > /dev/null 2>&1; then
        print_success "API is healthy!"
        curl -s http://localhost:3000/health | jq . 2>/dev/null || curl -s http://localhost:3000/health
    else
        print_error "API is not responding!"
        echo "Check service status: $0 status"
        echo "Check logs: $0 logs"
    fi
}

# Function to show metrics
show_metrics() {
    print_status "Fetching API metrics..."
    if curl -s -f http://localhost:3000/metrics > /dev/null 2>&1; then
        curl -s http://localhost:3000/metrics | jq . 2>/dev/null || curl -s http://localhost:3000/metrics
    else
        print_error "Could not fetch metrics!"
        echo "Check if the API is running: $0 status"
    fi
}

# Main script logic
case "${1:-help}" in
    start)
        check_root
        check_service_exists
        print_status "Starting ${SERVICE_NAME}..."
        systemctl start "$SERVICE_NAME"
        print_success "Service started!"
        ;;
    stop)
        check_root
        check_service_exists
        print_status "Stopping ${SERVICE_NAME}..."
        systemctl stop "$SERVICE_NAME"
        print_success "Service stopped!"
        ;;
    restart)
        check_root
        check_service_exists
        print_status "Restarting ${SERVICE_NAME}..."
        systemctl restart "$SERVICE_NAME"
        print_success "Service restarted!"
        ;;
    status)
        check_service_exists
        print_status "Service status:"
        systemctl status "$SERVICE_NAME" --no-pager -l
        ;;
    logs)
        check_service_exists
        print_status "Showing service logs (follow mode):"
        print_warning "Press Ctrl+C to exit log view"
        journalctl -u "$SERVICE_NAME" -f
        ;;
    logs-all)
        check_service_exists
        print_status "Showing all service logs:"
        journalctl -u "$SERVICE_NAME" --no-pager
        ;;
    enable)
        check_root
        check_service_exists
        print_status "Enabling ${SERVICE_NAME} to start on boot..."
        systemctl enable "$SERVICE_NAME"
        print_success "Service enabled!"
        ;;
    disable)
        check_root
        check_service_exists
        print_status "Disabling ${SERVICE_NAME} from starting on boot..."
        systemctl disable "$SERVICE_NAME"
        print_success "Service disabled!"
        ;;
    reload)
        check_root
        check_service_exists
        print_status "Reloading service configuration..."
        systemctl daemon-reload
        systemctl reload "$SERVICE_NAME" 2>/dev/null || print_warning "Service does not support reload, restarting instead..." && systemctl restart "$SERVICE_NAME"
        print_success "Configuration reloaded!"
        ;;
    health)
        check_health
        ;;
    metrics)
        show_metrics
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        show_usage
        exit 1
        ;;
esac
