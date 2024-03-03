import sys
from matplotlib import pyplot as plt

def main(filename):
    handle = open(filename, "r")
    txt = handle.read()
    data = [ln.split(":") for ln in txt.split("\n")][:-1]
    names, means, devs = zip(*data)
    means = list(map(float, means))
    devs = list(map(float, devs))
    positioning = [i * 0.5 for i in range(10)]
    plt.bar(positioning, means, width=[.25] * 10)
    plt.errorbar(positioning, means, yerr=devs,
                 fmt=".", color="r", linewidth=2,
                 capsize=3, capthick=2, markersize=1)
    plt.xticks(positioning, names)
    plt.xlabel("Game strategies")
    plt.ylabel("Average score")
    plt.legend(loc="best")
    plt.show()


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("error: usage is 'python3 graph_results.py tournament results' ")
        sys.exit(1)
    main(sys.argv[1])