// Loop and branch coverage for the HLSL frontend.
float accumulate(float start, int count)
{
    float total = start;

    for (int i = 0; i < count; i++)
    {
        total += float(i) * 2.0f;
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
        total = total - 0.5f;
    }

    do
    {
        total *= 0.5f;
        remaining++;
    } while (remaining < 2);

    return total;
}
