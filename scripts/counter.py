x = [295882, 4196, 713, 392, 874, 392, 874, 1034, 874, 392, 874, 392, 874, 391, 874, 392, 874, 391, 714, 391, 874, 392, 874, 391, 874, 392, 874, 392, 874, 391, 874, 392, 713, 392, 874, 391, 874, 1196, 874, 391, 714, 1195, 874, 392, 874, 1035, 874, 391, 874, 392, 874, 391, 874, 392, 874, 1035, 874, 391, 874, 392, 874, 392, 874, 391, 874, 1035, 874, 1196, 874, 391, 713, 392, 874, 1196, 874, 391, 874, 1035, 874]

y = [i // 160 for i in x]  # 使用列表推导式将每个元素除以 2
print(y)
