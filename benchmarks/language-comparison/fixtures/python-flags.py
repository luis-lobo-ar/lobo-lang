import sys


def valid(amount):
    return amount >= 0 and amount <= 100


def decode(kind, amount):
    if kind == 0:
        return (0, amount)
    if kind == 1:
        return (1, amount)
    if kind == 2:
        return (2, amount)
    return (-1, 0)


def step(state, action):
    kind, amount = state
    if kind == 0 and action == 0:
        return (1, amount)
    if kind == 1 and action == 1:
        return (2, amount)
    if kind == 2 and action == 2:
        return (0, amount)
    return (-1, 0)


def code(state):
    kind, amount = state
    if kind == 0:
        return amount
    if kind == 1:
        return 1000 + amount
    if kind == 2:
        return 2000 + amount
    return -900


def run(kind, amount, action):
    if not valid(amount):
        return -100
    return code(step(decode(kind, amount), action))


if __name__ == '__main__':
    print(run(*map(int, sys.argv[1:])))
