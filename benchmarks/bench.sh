#!/bin/bash
set -e

RUNS=5

BOLD='\033[1m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
RESET='\033[0m'
DIM='\033[2m'

declare -a NAMES
declare -a TIMES

run_benchmark() {
    local name="$1"
    shift
    local cmd=("$@")

    echo -e "${CYAN}▶ Benchmarking:${RESET} ${BOLD}$name${RESET} ${DIM}(${RUNS} runs)${RESET}"

    local total=0
    local all_times=()

    for i in $(seq 1 $RUNS); do
        local start
        start=$(date +%s%N)
        "${cmd[@]}" > /dev/null 2>&1
        local end
        end=$(date +%s%N)
        local elapsed_ns=$(( end - start ))
        local elapsed_ms=$(echo "scale=3; $elapsed_ns / 1000000" | bc)
        all_times+=("$elapsed_ms")
        total=$(echo "scale=3; $total + $elapsed_ms" | bc)
        echo -e "  ${DIM}Run $i: ${elapsed_ms} ms${RESET}"
    done

    local avg
    avg=$(echo "scale=3; $total / $RUNS" | bc)

    NAMES+=("$name")
    TIMES+=("$avg")

    echo -e "  ${GREEN}✓ Average: ${BOLD}${avg} ms${RESET}\n"
}

echo -e "\n${BOLD}${BLUE}═══════════════════════════════════════${RESET}"
echo -e "${BOLD}${BLUE}         COMPILATION PHASE              ${RESET}"
echo -e "${BOLD}${BLUE}═══════════════════════════════════════${RESET}\n"

mkdir -p build
cd build

echo -e "${CYAN}▶ Compiling C++...${RESET}"
g++ -O3 ../main.cxx -o cpp
echo -e "${GREEN}✓ Done${RESET}\n"

echo -e "${CYAN}▶ Building Cylium interpreter...${RESET}"
(cd ../../interpreter && cargo build --release)
echo -e "${GREEN}✓ Done${RESET}\n"

echo -e "${BOLD}${BLUE}═══════════════════════════════════════${RESET}"
echo -e "${BOLD}${BLUE}         BENCHMARK PHASE                ${RESET}"
echo -e "${BOLD}${BLUE}═══════════════════════════════════════${RESET}\n"

run_benchmark "C++"    ./cpp
run_benchmark "Python" python3 ../main.py
run_benchmark "Lua"    lua ../main.lua
run_benchmark "Cylium" ../../interpreter/target/release/cylium ../main.cyl

echo -e "${BOLD}${BLUE}═══════════════════════════════════════${RESET}"
echo -e "${BOLD}${BLUE}            RESULTS TABLE               ${RESET}"
echo -e "${BOLD}${BLUE}═══════════════════════════════════════${RESET}\n"

min_time=${TIMES[0]}
for t in "${TIMES[@]}"; do
    if (( $(echo "$t < $min_time" | bc -l) )); then
        min_time=$t
    fi
done

printf "${BOLD}  %-12s  %12s  %10s${RESET}\n" "Language" "Avg Time (ms)" "Relative"
echo -e "  ${DIM}────────────────────────────────────────${RESET}"

for i in "${!NAMES[@]}"; do
    name="${NAMES[$i]}"
    avg="${TIMES[$i]}"
    rel=$(echo "scale=2; $avg / $min_time" | bc)

    # Color by relative speed
    if (( $(echo "$rel <= 1.05" | bc -l) )); then
        color=$GREEN
    elif (( $(echo "$rel <= 3.0" | bc -l) )); then
        color=$YELLOW
    else
        color=$MAGENTA
    fi

    printf "  ${color}${BOLD}%-12s${RESET}  %12s ms  %9sx\n" \
        "$name" "$avg" "$rel"
done

echo -e "  ${DIM}────────────────────────────────────────${RESET}"
echo -e "\n  ${DIM}Each benchmark ran ${RUNS} times. Times shown are averages.${RESET}\n"