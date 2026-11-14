// Loop and branch coverage for the GLSL frontend.
float accumulate(float start, int count)
{
    float total = start;

    for (int i = 0; i < count; ++i)
    {
        total += float(i) * 2.0;
    }

    int remaining = count;
    while (remaining > 0)
    {
        remaining--;
        if (remaining == 3)
        {
            continue;
        }
        else if (remaining == 1)
        {
            break;
        }
        total = total - 0.5;
    }

    do
    {
        total *= 0.5;
        remaining++;
    } while (remaining < 2);

    return total;
}
