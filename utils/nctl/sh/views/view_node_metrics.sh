#!/usr/bin/env bash

source $NCTL/sh/utils/main.sh

#######################################
# Renders chain height at specified node(s).
# Arguments:
#   Node ordinal identifier.
#   Metric identifier.
#######################################
function main()
{
    local NODE_ID=${1}
    local METRIC=${2}

    if [ $NODE_ID = "all" ]; then
        for NODE_ID in $(seq 1 $(get_count_of_nodes))
        do
            do_render $NODE_ID $METRIC
        done
    else
        do_render $NODE_ID $METRIC
    fi
}

#######################################
# Displays to stdout current node metrics.
# Arguments:
#   Network ordinal identifier.
#   Node ordinal identifier.
#   Metric name.
#######################################
function do_render()
{
    local NODE_ID=${1}
    local METRICS=${2}

    local ENDPOINT=$(get_node_address_rest $NODE_ID)/metrics

    if [ $METRICS = "all" ]; then
        curl -s --location --request GET $ENDPOINT  
    else
        echo "node #$NODE_ID :: "$(curl -s --location --request GET $ENDPOINT | grep $METRICS | tail -n 1)
    fi
}

# ----------------------------------------------------------------
# ENTRY POINT
# ----------------------------------------------------------------

unset NODE_ID
unset METRIC

for ARGUMENT in "$@"
do
    KEY=$(echo $ARGUMENT | cut -f1 -d=)
    VALUE=$(echo $ARGUMENT | cut -f2 -d=)
    case "$KEY" in
        metric) METRIC=${VALUE} ;;
        node) NODE_ID=${VALUE} ;;
        *)
    esac
done

METRIC=${METRIC:-"all"}
NODE_ID=${NODE_ID:-"all"}

main \
    ${NODE_ID:-"all"} \
    ${METRIC:-"all"}
